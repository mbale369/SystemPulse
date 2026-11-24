"""Verify the exact official glib archive and the two-line local backport; no network or writes."""
import argparse
import hashlib
from pathlib import Path, PurePosixPath
import tarfile
import tomllib

CHECKSUM = '233daaf6e83ae6a12a52055f568f9d7cf4671dabb78ff9560ab6da230ce00ee5'


def verify(archive: Path, root: Path):
    if archive.stat().st_size > 1024 * 1024:
        raise ValueError('Unexpected archive size')
    if hashlib.sha256(archive.read_bytes()).hexdigest() != CHECKSUM:
        raise ValueError('Official archive checksum mismatch')
    vendor = root / 'vendor/glib-0.18.5'
    expected = set()
    changed = []
    with tarfile.open(archive, 'r:gz') as package:
        for item in package.getmembers():
            if not item.isfile():
                if item.issym() or item.islnk():
                    raise ValueError('Unexpected archive link')
                continue
            relative = PurePosixPath(item.name).relative_to('glib-0.18.5')
            if '..' in relative.parts or item.size > 2 * 1024 * 1024:
                raise ValueError('Unsafe archive entry')
            name = str(relative)
            expected.add(name)
            original = package.extractfile(item).read()
            wanted = original
            if name == 'src/variant_iter.rs':
                old = b'let p: *mut libc::c_char = std::ptr::null_mut();'
                if wanted.count(old) != 1 or wanted.count(b'                &p,') != 1:
                    raise ValueError('Unexpected upstream iterator source')
                wanted = wanted.replace(old, b'let mut p: *mut libc::c_char = std::ptr::null_mut();', 1)
                wanted = wanted.replace(b'                &p,', b'                &mut p,', 1)
                changed.append(name)
            local = vendor / name
            if local.is_symlink() or local.read_bytes() != wanted:
                raise ValueError(f'Vendor content mismatch: {name}')
    actual = {str(p.relative_to(vendor)) for p in vendor.rglob('*') if p.is_file()}
    if actual != expected | {'SYSTEMPULSE_PATCH.md'}:
        raise ValueError('Unexpected vendor files')
    manifest = tomllib.loads((root / 'apps/desktop/src-tauri/Cargo.toml').read_text())
    if manifest['patch']['crates-io']['glib']['path'] != '../../../vendor/glib-0.18.5':
        raise ValueError('Local patch not wired')
    if manifest['profile']['dev']['package']['glib']['opt-level'] != 2:
        raise ValueError('Optimized regression profile missing')
    print(f'PASS official archive SHA-256; {len(expected)} original files verified; only two official source-line changes; original license/version preserved; patch wired; optimized test profile')
    print('patched_variant_iter_sha256=' + hashlib.sha256((vendor / 'src/variant_iter.rs').read_bytes()).hexdigest())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True, help='Official glib-0.18.5.crate from crates.io or Cargo cache')
    args = parser.parse_args()
    verify(args.archive, Path(__file__).resolve().parents[1])


if __name__ == '__main__':
    main()
