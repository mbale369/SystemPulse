import importlib.util
import os
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('installer', Path(__file__).resolve().parents[1] / 'scripts/install-user.py')
installer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(installer)


class InstallationTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix='systempulse-install-test-'))
        self.home = self.root / 'Home with spaces'
        self.home.mkdir(mode=0o700)
        self.sources = self.root / 'build'
        self.sources.mkdir()
        for name in ['systempulse-desktop', 'systempulse-mcp', 'icon.png']:
            (self.sources / name).write_bytes(b'isolated test fixture')

    def install(self, version='v1', startup='disable', **kw):
        return installer.install_user(self.home, self.sources, self.sources / 'icon.png', version, startup, **kw)

    def test_default_install_is_not_autostart_and_never_touches_agents(self):
        private = self.home / '.codex'
        private.mkdir()
        (private / 'config.toml').write_text('test sentinel, not a credential')
        self.install()
        self.assertFalse((self.home / '.config/autostart').exists())
        self.assertIn('exec ', (self.home / '.local/bin/systempulse-desktop').read_text())
        self.assertEqual((private / 'config.toml').read_text(), 'test sentinel, not a credential')

    def test_explicit_enable_disable_preserves_old_version_and_backups(self):
        self.install(startup='enable')
        startup = self.home / '.config/autostart/io.systempulse.desktop.desktop'
        self.assertIn('--widget', startup.read_text())
        self.assertIn('Hidden=false', startup.read_text())
        self.install('v2', 'disable', update=True)
        self.assertIn('Hidden=true', startup.read_text())
        self.assertTrue((self.home / '.local/share/systempulse/v1/systempulse-desktop').exists())
        self.assertTrue((self.home / '.local/share/systempulse/v2/previous-launcher-0.txt').exists())

    def test_dry_run_and_repeated_version_never_overwrite(self):
        self.install(dry_run=True)
        self.assertFalse((self.home / '.local').exists())
        self.install()
        with self.assertRaises(ValueError): self.install(update=True)

    def test_unmanaged_launcher_and_symlink_are_refused(self):
        path = self.home / '.local/bin'
        path.mkdir(parents=True)
        (path / 'systempulse-desktop').write_text('unrelated user file')
        with self.assertRaises(ValueError): self.install(update=True)
        self.assertEqual((path / 'systempulse-desktop').read_text(), 'unrelated user file')
        other = self.home / '.config'
        other.symlink_to(self.sources, target_is_directory=True)
        with self.assertRaises(ValueError): self.install(update=True)

    def test_input_paths_and_permissions_are_validated(self):
        for value in ['../escape', '..', '', 'v/1']:
            with self.assertRaises(ValueError): self.install(value)
        for value in ['enable\nExec=bad','ask']:
            with self.assertRaises(ValueError): self.install(startup=value)
        with self.assertRaises(ValueError): installer.desktop_quote(Path('/tmp/evil%f'))
        self.assertEqual(installer.desktop_quote(Path('/tmp/space name')), '"/tmp/space name"')
        os.chmod(self.home, 0o777)
        with self.assertRaises(ValueError): self.install()

    def test_private_ancestor_protects_group_writable_children_without_chmod(self):
        local=self.home / '.local'
        local.mkdir(mode=0o700)
        directory=local / 'bin'
        directory.mkdir(mode=0o775)
        os.chmod(directory,0o775)
        self.install()
        self.assertEqual(directory.stat().st_mode & 0o777,0o775)

    def test_visible_default_choice_and_capsule_only_launcher(self):
        self.assertEqual(installer.default_autostart_choice(''),'enable')
        self.assertEqual(installer.default_autostart_choice('oui'),'enable')
        self.assertEqual(installer.default_autostart_choice('n'),'disable')
        self.assertEqual(installer.default_autostart_choice('unexpected'),'disable')
        self.install(startup='enable')
        app=self.home / '.local/share/applications/io.systempulse.desktop.desktop'
        self.assertIn(' --widget\n',app.read_text())
        self.assertNotIn('--dashboard',app.read_text())


if __name__ == '__main__':
    unittest.main()
