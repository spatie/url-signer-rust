"""Test configuration through its command-line interface, without requiring Cargo."""

import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

TEMPLATE = Path(__file__).resolve().parents[1]


class ConfigureTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name) / "sample-rust"
        shutil.copytree(
            TEMPLATE,
            self.root,
            ignore=shutil.ignore_patterns("target", "__pycache__", ".git"),
        )

    def configure(self, *arguments, answers=None):
        return subprocess.run(
            [sys.executable, str(self.root / "configure.py"), "--no-check", *arguments],
            cwd=self.directory.name,
            input=answers,
            capture_output=True,
            text=True,
            encoding="utf-8",
            env={**os.environ, "PYTHONIOENCODING": "utf-8"},
        )

    def snapshot(self):
        return {
            str(path.relative_to(self.root)): path.read_bytes()
            for path in self.root.rglob("*")
            if path.is_file()
        }

    def test_configures_metadata_imports_docs_and_cleanup(self):
        expected_headings = [
            "Support us",
            "Installation",
            "Usage",
            "Testing",
            "Changelog",
            "Contributing",
            "Security Vulnerabilities",
            "Credits",
            "License",
        ]
        template_readme = (self.root / "README.md").read_text(encoding="utf-8")
        self.assertEqual(
            re.findall(r"^## (.+)$", template_readme, re.MULTILINE),
            expected_headings,
        )
        description = 'Work with "images" at C:\\photos: café'
        author = 'Zoë "Example"'
        result = self.configure(
            "--defaults",
            "--crate-name",
            "spatie-photo-tools",
            "--repository",
            "photo-tools-rust",
            "--owner",
            "example-org",
            "--author-name",
            author,
            "--author-email",
            "zoe@example.org",
            "--author-username",
            "zoe-example",
            "--description",
            description,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        manifest = tomllib.loads(
            (self.root / "Cargo.toml").read_text(encoding="utf-8")
        )["package"]
        self.assertEqual(manifest["name"], "spatie-photo-tools")
        self.assertEqual(manifest["authors"], [f"{author} <zoe@example.org>"])
        self.assertEqual(manifest["description"], description)
        self.assertEqual(
            manifest["repository"], "https://github.com/example-org/photo-tools-rust"
        )
        self.assertEqual(manifest["edition"], "2024")
        self.assertEqual(manifest["rust-version"], "1.89")
        self.assertIn(
            "use spatie_photo_tools::greet",
            (self.root / "tests/greeting.rs").read_text(),
        )
        self.assertIn("title: v1\n", (self.root / "docs/_index.md").read_text())
        self.assertIn(
            "github: example-org", (self.root / ".github/FUNDING.yml").read_text()
        )
        self.assertIn(
            "${{ matrix.os }}", (self.root / ".github/workflows/ci.yml").read_text()
        )
        for name in ("configure.py", "SKELETON.md", "tests/test_configure.py"):
            self.assertFalse((self.root / name).exists(), name)
        readme = (self.root / "README.md").read_text(encoding="utf-8")
        self.assertNotIn("<!--delete-->", readme)
        self.assertNotIn("<!--/delete-->", readme)
        self.assertNotIn("python3 configure.py", readme)
        self.assertIn(
            "https://github.com/apollographql/skills/tree/main/skills/rust-best-practices",
            readme,
        )
        self.assertIn(
            "https://spatie.be/github-ad-image/photo-tools-rust.jpg", readme
        )
        self.assertIn("https://spatie.be/github-ad-click/photo-tools-rust", readme)
        self.assertEqual(
            re.findall(r"^## (.+)$", readme, re.MULTILINE),
            expected_headings,
        )
        self.assertTrue(readme.startswith("# spatie-photo-tools\n\n[![Latest version"))
        self.assertLess(readme.index("[![Total downloads]"), readme.index(description))
        self.assertLess(readme.index(description), readme.index("## Support us"))
        for name, value in self.snapshot().items():
            self.assertNotRegex(value, rb"\{\{[a-z_]+\}\}", name)

    def test_interactive_configuration_reprompts_and_confirms(self):
        result = self.configure(
            answers="Invalid Name\nspatie-interactive\ninteractive-rust\nspatie\nFreek\nfreek@example.org\nfreekmurze\nAn interactive package\ny\n"
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Use a lowercase Cargo name", result.stdout)
        self.assertIn(
            'name = "spatie-interactive"', (self.root / "Cargo.toml").read_text()
        )

    def test_declining_and_eof_leave_files_unchanged(self):
        before = self.snapshot()
        for answers in ("\n" * 7 + "n\n", ""):
            with self.subTest(answers=answers):
                result = self.configure(answers=answers)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.snapshot(), before)

    def test_invalid_names_and_multiline_metadata_leave_files_unchanged(self):
        before = self.snapshot()
        for flag, value in (
            ("--crate-name", "gen"),
            ("--crate-name", "123-crate"),
            ("--crate-name", "con"),
            ("--crate-name", "a" * 65),
            ("--repository", "../escape"),
            ("--owner", "not/owner"),
            ("--description", "line one\nline two"),
            ("--description", "{{crate_name}}"),
        ):
            with self.subTest(flag=flag, value=value):
                result = self.configure("--defaults", flag, value)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertEqual(self.snapshot(), before)

    def test_unknown_placeholder_fails_before_any_writes(self):
        (self.root / "unexpected.md").write_text("{{unknown_placeholder}}")
        before = self.snapshot()
        result = self.configure("--defaults")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.snapshot(), before)

    def test_does_not_change_git_build_files_or_binary_assets(self):
        for name in (".git/config", "target/cache", "asset.bin"):
            path = self.root / name
            path.parent.mkdir(exist_ok=True)
            path.write_bytes(
                b"\xff{{crate_name}}" if name.endswith("bin") else b"{{crate_name}}"
            )
        before = {
            name: (self.root / name).read_bytes()
            for name in (".git/config", "target/cache", "asset.bin")
        }
        result = self.configure("--defaults")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            {name: (self.root / name).read_bytes() for name in before}, before
        )


if __name__ == "__main__":
    unittest.main()
