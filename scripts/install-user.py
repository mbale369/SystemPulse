"""Linux user installation only. No sudo, dependency installation, client config or implicit autostart."""
import argparse
import datetime
import os
from pathlib import Path
import re
import shlex
import shutil
import sys
import tempfile


def safe_target(home: Path, target: Path):
    home = home.absolute()
    if not target.is_absolute() or '..' in target.parts:
        raise ValueError('Unsafe installation path')
    relative = target.relative_to(home)
    current = home
    private_ancestor = False
    for part in (None, *relative.parts):
        if part is not None:
            current = current / part
        if current.is_symlink():
            raise ValueError('Symbolic installation paths are refused')
        if current.exists():
            metadata = current.stat()
            # A 0700 owned ancestor already excludes other users, including group
            # members. Do not chmod a pre-existing ~/.local/bin just to install.
            if metadata.st_uid != os.getuid() or (metadata.st_mode & 0o022 and not private_ancestor):
                raise ValueError('Installation path ownership or permissions are unsafe')
            if current.is_dir() and metadata.st_mode & 0o077 == 0:
                private_ancestor = True


def desktop_quote(path: Path):
    value = str(path)
    if any(c in value for c in '\n\r\x00%"`$\\'):
        raise ValueError('Unsupported control or desktop Exec characters in path')
    return '"' + value + '"'


def existing_text(path: Path, kind: str):
    if not path.exists():
        return None
    if not path.is_file() or path.stat().st_size > 16_384:
        raise ValueError('Refusing a nonstandard existing launcher')
    text = path.read_text(encoding='utf-8')
    owned = ('# SystemPulse managed launcher' in text or '# Local user installation; embedded frontend' in text) if kind == 'launcher' else (
        'X-SystemPulse-Managed=true' in text or ('Name=SystemPulse\n' in text and 'systempulse-desktop\n' in text and '[Desktop Entry]' in text))
    if not owned:
        raise ValueError('Existing file is not a recognized SystemPulse launcher')
    return text


def atomic_text(path: Path, text: str, mode=0o644):
    fd, name = tempfile.mkstemp(prefix='.systempulse-pending-', dir=path.parent)
    # Only a newly-created temporary is written; os.replace is on the same filesystem.
    # On failure the private temporary is retained rather than deleting unknown files.
    with os.fdopen(fd, 'w', encoding='utf-8') as stream:
        os.fchmod(stream.fileno(), mode)
        stream.write(text)
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(name, path)
    directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


def ensure_directory(home: Path, path: Path):
    current = home
    for part in path.relative_to(home).parts:
        current = current / part
        if not current.exists():
            current.mkdir(mode=0o755)


def install_user(home: Path, binaries: Path, icon: Path, version: str, autostart: str, update=False, dry_run=False, mcp_binary=None):
    if not re.fullmatch(r'[a-zA-Z0-9._-]{1,80}', version) or version in ('.', '..'):
        raise ValueError('Invalid installation version')
    if autostart not in ('enable', 'disable', 'keep'):
        raise ValueError('Explicit autostart decision required')
    home = home.absolute()
    destination = home / '.local/share/systempulse' / version
    launcher = home / '.local/bin/systempulse-desktop'
    application = home / '.local/share/applications/io.systempulse.desktop.desktop'
    startup = home / '.config/autostart/io.systempulse.desktop.desktop'
    for path in [destination, launcher, application, startup]:
        safe_target(home, path)
    if destination.exists():
        raise ValueError('Installation version already exists; nothing overwritten')
    previous = [(launcher, existing_text(launcher, 'launcher')), (application, existing_text(application, 'desktop'))]
    if startup.exists():
        previous.append((startup, existing_text(startup, 'desktop')))
    if not update and any(text is not None for _, text in previous):
        raise ValueError('An installation exists; explicit --update required')
    sources = [binaries / 'systempulse-desktop', mcp_binary or binaries / 'systempulse-mcp', icon]
    for source in sources:
        if source.is_symlink() or not source.is_file() or source.stat().st_size > 300 * 1024 * 1024:
            raise ValueError('Missing, symbolic or oversized installation source')
    executable = destination / 'systempulse-desktop'
    quoted = desktop_quote(launcher)
    desktop = ('[Desktop Entry]\nVersion=1.0\nType=Application\nName=SystemPulse\n'
               'Comment=Local system health monitor\nTerminal=false\nCategories=System;Monitor;\n'
               f'Exec={quoted} --widget\nIcon={destination / "icon.png"}\nStartupWMClass=io.systempulse.desktop\n'
               'StartupNotify=true\nX-SystemPulse-Managed=true\n')
    startup_text = desktop + ('Hidden=false\n' if autostart == 'enable' else 'Hidden=true\n')
    plan = {'version': version, 'destination': str(destination), 'autostart': autostart, 'dry_run': dry_run}
    if dry_run:
        return plan
    for path in [destination.parent, launcher.parent, application.parent]:
        ensure_directory(home, path)
    if autostart == 'enable' or startup.exists():
        ensure_directory(home, startup.parent)
    destination.mkdir(mode=0o700)
    for source, name in zip(sources, ['systempulse-desktop', 'systempulse-mcp', 'icon.png']):
        shutil.copyfile(source, destination / name)
        os.chmod(destination / name, 0o644 if name == 'icon.png' else 0o755)
    for i, (_, text) in enumerate(previous):
        if text is not None:
            atomic_text(destination / f'previous-launcher-{i}.txt', text, 0o600)
    atomic_text(application, desktop)
    if autostart != 'keep' and (autostart == 'enable' or startup.exists()):
        atomic_text(startup, startup_text)
    # Switch the command only after all resources are installed. No process is started here.
    atomic_text(launcher, '#!/bin/sh\n# SystemPulse managed launcher\nexec ' + shlex.quote(str(executable)) + ' "$@"\n', 0o755)
    return plan


def default_autostart_choice(answer: str):
    # The preselected Yes is visible in the interactive prompt, never inferred
    # from EOF or --yes. An unknown answer fails closed to disabled.
    return 'enable' if answer.strip().lower() in ('', 'o', 'oui', 'y', 'yes') else 'disable'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--yes', action='store_true', help='Explicitly approve installation (does not approve autostart)')
    parser.add_argument('--update', action='store_true', help='Replace recognized launchers only; preserve old version and backups')
    parser.add_argument('--dry-run', action='store_true')
    parser.add_argument('--autostart', choices=['ask', 'enable', 'disable', 'keep'], default='ask')
    parser.add_argument('--version', default='0.1.0-local-widget-' + datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%d-%H%M%S'))
    args = parser.parse_args()
    if sys.platform != 'linux' or not hasattr(os, 'getuid') or os.getuid() == 0:
        parser.error('Linux user account required; never use sudo')
    if not args.yes and not args.dry_run:
        if input('Installer SystemPulse pour votre compte ? [o/N] ').strip().lower() not in ('o', 'oui', 'y', 'yes'):
            parser.error('Installation non autorisée')
    choice = args.autostart
    if choice == 'ask':
        if args.dry_run:
            choice = 'enable'
        else:
            try:
                answer = input('Lancer la capsule seule à chaque connexion au bureau ? [O/n] (modifiable) ').strip().lower()
            except EOFError:
                parser.error('Consentement visible requis : préciser --autostart enable ou disable en mode non interactif')
            choice = default_autostart_choice(answer)
    root = Path(__file__).resolve().parents[1]
    import json
    try:
        print(json.dumps(install_user(Path.home(), root / 'apps/desktop/src-tauri/target/debug', root / 'apps/desktop/src-tauri/icons-power/128x128@2x.png', args.version, choice, args.update, args.dry_run, mcp_binary=root / 'target/debug/systempulse-mcp')))
    except (OSError, ValueError) as error:
        parser.exit(1, f'Installation refusée ou non confirmée : {type(error).__name__}. Aucun succès annoncé.\n')


if __name__ == '__main__':
    main()
