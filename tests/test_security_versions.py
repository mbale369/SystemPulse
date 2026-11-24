"""Offline regression guards; these do not replace a current RustSec database audit."""
import hashlib
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


def load(path):
    return tomllib.loads((ROOT / path).read_text())


class SecurityVersionTests(unittest.TestCase):
    def test_fixed_registry_versions_are_in_lockfiles(self):
        cases = [('Cargo.lock', 'rmcp', (3, 9, 0)),
                 ('apps/desktop/src-tauri/Cargo.lock', 'url', (2, 5, 9)),
                 ('apps/desktop/src-tauri/Cargo.lock', 'cookie', (0, 18, 3))]
        for path, name, minimum in cases:
            versions = [p['version'] for p in load(path)['package'] if p['name'] == name]
            self.assertTrue(versions, name)
            for version in versions:
                self.assertGreaterEqual(tuple(map(int, version.split('.')[:3])), minimum, name)

    def test_mcp_stays_local_server_without_http_client_features(self):
        dep = load('crates/mcp-server/Cargo.toml')['dependencies']['rmcp']
        self.assertFalse(dep['default-features'])
        self.assertEqual(set(dep['features']), {'server', 'transport-io', 'schemars'})

    def test_local_glib_backport_is_exact_and_optimized(self):
        source = ROOT / 'vendor/glib-0.18.5/src/variant_iter.rs'
        self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(),
                         'a0f5ee8acb8faa089bcdfbc9a57372609fce7654026ccef7d9a224d05a654ccc')
        desktop = load('apps/desktop/src-tauri/Cargo.toml')
        self.assertEqual(desktop['patch']['crates-io']['glib']['path'], '../../../vendor/glib-0.18.5')
        self.assertEqual(desktop['profile']['dev']['package']['glib']['opt-level'], 2)
        self.assertEqual(load('vendor/glib-0.18.5/Cargo.toml')['package']['version'], '0.18.5')
        self.assertTrue((ROOT / 'vendor/glib-0.18.5/LICENSE').is_file())

    def test_no_advisory_ignore_was_added(self):
        self.assertEqual(load('deny.toml').get('advisories', {}).get('ignore', []), [])


if __name__ == '__main__':
    unittest.main()
