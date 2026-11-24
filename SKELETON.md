# Rust package skeleton

This template creates a standalone Rust library with Cargo tooling and Spatie's documentation layout. Python 3.11+ is needed only to configure the template; the resulting crate has no Python or third-party Rust dependencies.

## Configure a new package

Copy the template or use GitHub's "Use this template" button once a repository exists. Install [Rust](https://rustup.rs/) and Python 3.11+, then run:

```sh
python3 configure.py
```

On Windows use `py -3 configure.py`. Author name and email default to Git configuration when available. Crate names and repository names are separate: `spatie-example` can live in `spatie/example-rust` and is imported as `spatie_example`.

For automation:

```sh
python3 configure.py --defaults --crate-name spatie-example \
  --repository example-rust --owner spatie \
  --author-name 'Freek Van der Herten' --author-email freek@spatie.be \
  --author-username freekmurze --description 'Describe the package here'
```

`--defaults` accepts the supplied values and remaining defaults without confirmation. The interactive path shows a summary and requires confirmation. `--no-check` configures without invoking Cargo. Configuration writes no Git history and performs no publication, pushes, repository creation or dependency installation. It removes this file, its own script and its Python tests. It then generates Cargo.lock and runs format, Clippy and tests unless `--no-check` was given. Cargo check failures return a nonzero exit status; the configured files remain available to fix. When skipping checks, generate Cargo.lock before using `--locked` commands.

## Start the real library

Replace `greet` and its examples/tests. Edit Cargo keywords and categories. Set the package version as appropriate.

The README ad image and click URL use the configured repository name, such as `period-rust`. Arrange a matching ad image on spatie.be before publishing; unregistered names return 404.

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html): snake_case functions, conventional conversions, documented public items, private fields, `Debug` on public types, and typed errors implementing `std::error::Error` when an API can fail. Add `# Errors`, `# Panics` and `# Safety` sections only where relevant. Do not add error types, builders or traits to an API that does not need them.

## Documentation

`docs/_index.md` declares `title: v1`, `githubUrl`, and `branch: main`. Other pages use `title` and `weight` frontmatter. This matches Spatie's Rust ports. Website registration and hosting are separate from creating the package.

The crate includes README.md as its root rustdoc. build.rs discovers Markdown guides recursively and includes them in doctest-only modules. Use executable `rust` fences, assertions and rustdoc hidden `# ` lines for setup. Do not use `ignore` or `no_run` for examples that are supposed to execute. Hidden setup is hidden in rustdoc; the website renderer must also support rustdoc hidden lines before the guides are published there.

The docs and build script must stay in the Cargo package because downstream builds and doctests use them. `cargo package` in CI verifies that the distributed crate builds, not only the checkout.

## Automation

- CI runs stable Rust on Linux, macOS and Windows, with rustfmt, Clippy denying warnings, tests, executable documentation and warning-free rustdoc.
- A Linux job tests Rust 1.89.0. Keep Cargo.lock committed so CI audits and tests the same dependency resolution. Review MSRV compatibility when adding dependencies.
- A separate job runs cargo audit. Dependabot updates Cargo dependencies and GitHub Actions weekly.
- The template itself runs its Python configurator regression tests in the OS matrix before creating a sample crate. These steps are no-ops after configuration.
- Formatting is checked, not committed by CI. Dependency updates are reviewed, not auto-merged. Changelog changes are written by maintainers, not a release-triggered write workflow.
- No publishing workflow is supplied. Before a release, update CHANGELOG.md, run the documented checks, review `cargo package --list` and publish only when authorized. Enable required status checks on the new repository.

## Maintaining the template

```sh
python3 -m unittest discover -s tests -p 'test_configure.py' -v
```

Template placeholders use `{{name}}`; configuration fails before writing if it finds an unknown placeholder. JSON-quoted substitutions provide valid TOML and YAML strings for author details and descriptions containing quotes or backslashes. Git, build artifacts and binary assets are not rewritten.
