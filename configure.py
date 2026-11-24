#!/usr/bin/env python3
"""Configure a new Rust package using only the Python 3.11+ standard library."""

import argparse
import datetime
import json
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent
TOKEN = re.compile(r"\{\{([a-z_]+)\}\}")
SKIP = {".git", "target", "__pycache__", ".venv"}
REMOVE = ("configure.py", "tests/test_configure.py", "SKELETON.md")
RESERVED = set(
    "as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while abstract become box do final gen macro override priv typeof unsized virtual yield try union test core std alloc proc_macro".split()
)
RESERVED.update({"con", "prn", "aux", "nul", "com0", "lpt0"})
RESERVED.update(
    f"{prefix}{number}" for prefix in ("com", "lpt") for number in range(1, 10)
)


def git_config(key):
    if not shutil.which("git"):
        return ""
    result = subprocess.run(
        ["git", "config", "--get", key], cwd=ROOT, capture_output=True, text=True
    )
    return result.stdout.strip() if result.returncode == 0 else ""


def valid_crate(value):
    return (
        re.fullmatch(r"[a-z][a-z0-9]*(?:[-_][a-z0-9]+)*", value) is not None
        and len(value) <= 64
        and value.replace("-", "_") not in RESERVED
    )


def text(value):
    return (
        bool(value.strip())
        and not any(ord(char) < 32 or ord(char) == 127 for char in value)
        and "{{" not in value
        and "}}" not in value
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--defaults", action="store_true", help="Accept defaults without prompting"
    )
    parser.add_argument(
        "--no-check",
        action="store_true",
        help="Skip running Cargo checks after configuration",
    )
    fields = [
        (
            "crate_name",
            "Crate name",
            "spatie-skeleton",
            valid_crate,
            "Use a lowercase Cargo name, not a Rust keyword (maximum 64 characters).",
        ),
        (
            "repository",
            "GitHub repository name",
            ROOT.name if ROOT.name != "package-skeleton-rust" else "skeleton-rust",
            lambda value: (
                re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", value) is not None
                and len(value) <= 100
            ),
            "Use a GitHub repository name without an owner or URL.",
        ),
        (
            "owner",
            "GitHub owner",
            "spatie",
            lambda value: (
                re.fullmatch(r"[A-Za-z0-9](?:[A-Za-z0-9-]{0,37}[A-Za-z0-9])?", value)
                is not None
                and "--" not in value
            ),
            "Use a GitHub user or organization name.",
        ),
        (
            "author_name",
            "Author name",
            git_config("user.name") or "Spatie",
            text,
            "Enter a nonempty single-line name.",
        ),
        (
            "author_email",
            "Author email",
            git_config("user.email") or "info@spatie.be",
            lambda value: (
                text(value)
                and re.fullmatch(r"[^\s<>@]+@[^\s<>@]+\.[^\s<>@]+", value) is not None
            ),
            "Enter an email address.",
        ),
        (
            "author_username",
            "Author GitHub username",
            "freekmurze",
            lambda value: (
                re.fullmatch(r"[A-Za-z0-9](?:[A-Za-z0-9-]{0,37}[A-Za-z0-9])?", value)
                is not None
                and "--" not in value
            ),
            "Use a GitHub username.",
        ),
        (
            "description",
            "Package description",
            "A Rust package by Spatie",
            text,
            "Enter a nonempty single-line description.",
        ),
    ]
    for name, label, default, validator, hint in fields:
        parser.add_argument(
            "--" + name.replace("_", "-"), help=f"{label} (default: {default})"
        )
    args = parser.parse_args()
    values = {}
    for name, label, default, validator, hint in fields:
        value = getattr(args, name)
        if value is not None or args.defaults:
            value = default if value is None else value
            if not validator(value):
                parser.error(f"{label}: {hint}")
        else:
            while True:
                value = input(f"{label} [{default}]: ").strip() or default
                if validator(value):
                    break
                print(hint)
        values[name] = value

    print("\nConfigure this package:")
    for name, value in values.items():
        print(f"  {name.replace('_', ' ')}: {value}")
    if not args.defaults and input(
        "Modify template files? [y/N]: "
    ).strip().lower() not in {"y", "yes"}:
        print("Cancelled. No files changed.")
        return 1

    values["crate_import"] = values["crate_name"].replace("-", "_")
    values["year"] = str(datetime.date.today().year)
    values["author"] = f"{values['author_name']} <{values['author_email']}>"
    for name, value in list(values.items()):
        values[name + "_quoted"] = json.dumps(value, ensure_ascii=False)

    # Render and validate everything before changing any file.
    changes = {}
    for path in sorted(ROOT.rglob("*")):
        relative = path.relative_to(ROOT)
        if (
            any(part in SKIP for part in relative.parts)
            or str(relative).replace("\\", "/") in REMOVE
            or path.is_dir()
        ):
            continue
        if path.is_symlink():
            raise ValueError(f"Refusing to configure a symlink: {relative}")
        try:
            original = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        rendered = TOKEN.sub(lambda match: values[match[1]], original)
        rendered = re.sub(
            r"<!--delete-->.*?<!--/delete-->\n?",
            "",
            rendered,
            flags=re.DOTALL,
        )
        if rendered != original:
            changes[path] = rendered
    tomllib.loads(changes.get(ROOT / "Cargo.toml", (ROOT / "Cargo.toml").read_text()))
    for path, rendered in changes.items():
        path.write_text(rendered, encoding="utf-8", newline="\n")
    for name in REMOVE:
        (ROOT / name).unlink(missing_ok=True)

    print(
        "\nConfigured. The configure script and template-only files have been removed."
    )
    print("Review Cargo keywords/categories and the sample API before publishing.")
    if args.no_check:
        print(
            "Checks skipped. Run cargo generate-lockfile, then the commands in docs/testing.md."
        )
        return 0
    commands = [
        ["cargo", "generate-lockfile"],
        ["cargo", "fmt", "--all", "--", "--check"],
        [
            "cargo",
            "clippy",
            "--locked",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
        ["cargo", "test", "--locked", "--all-targets", "--all-features"],
        ["cargo", "test", "--locked", "--doc", "--all-features"],
    ]
    for command in commands:
        print("+ " + " ".join(command), flush=True)
        subprocess.run(command, cwd=ROOT, check=True)
    print(
        "Cargo checks passed. CI also checks rustdoc, MSRV, packaging and security advisories."
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (EOFError, KeyboardInterrupt):
        sys.exit("Cancelled.")
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        sys.exit(f"Configuration failed: {error}")
