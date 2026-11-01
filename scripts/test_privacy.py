#!/usr/bin/env python3
# Copyright (C) 2026 slatkisasa
# SPDX-License-Identifier: AGPL-3.0-only
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from check_privacy import findings

SCANNER = Path(__file__).with_name("check_privacy.py").resolve()


class PrivacyTests(unittest.TestCase):
    def test_email_and_paths(self):
        self.assertEqual(list(findings("test.cs", b"main@example.invalid dev@users.noreply.github.com")), [])
        self.assertTrue(list(findings("test.cs", b"user@" + b"private.test")))
        self.assertTrue(list(findings("test.cs", b"/Users/" + b"someone/project")))
        self.assertTrue(list(findings("test.cs", b"C:\\Users\\" + b"someone\\file")))

    def test_private_filenames(self):
        for name in ["accounts.json", "accounts.json.damaged-test", "Battle.net.config", "Battle.net.config.x.backup", "Backups/file"]:
            self.assertTrue(list(findings(name, b"{}")))
        self.assertEqual(list(findings("image.png", b"\0user@" + b"private.test")), [])

    def test_index_instead_of_worktree_and_history(self):
        with tempfile.TemporaryDirectory() as folder:
            def git(*args):
                return subprocess.check_output(["git", "-C", folder, *args], stderr=subprocess.DEVNULL)
            def scan(*args):
                return subprocess.run([os.sys.executable, str(SCANNER), *args], cwd=folder, capture_output=True)
            git("init")
            file = Path(folder) / "sample.txt"
            file.write_text("person@" + "private.test")
            git("add", ".")
            file.write_text("safe@example.invalid")
            result = scan()
            self.assertEqual(result.returncode, 1)
            self.assertNotIn(b"person@", result.stdout)
            git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "-c", "core.hooksPath=/dev/null", "commit", "-m", "fixture")
            git("add", ".")
            self.assertEqual(scan().returncode, 0)
            self.assertEqual(scan("--history").returncode, 1)


if __name__ == "__main__":
    unittest.main()
