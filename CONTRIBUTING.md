# Contributing

Please read [Spatie's contribution guidelines](https://github.com/spatie/.github/blob/main/CONTRIBUTING.md).

For Rust changes, run the commands in [docs/testing.md](docs/testing.md). Include a regression test for a bug fix and update the Markdown examples when changing the public API. Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/). Public types should implement `Debug`; fallible operations should return typed errors rather than panic on expected failures.

Keep Cargo.lock committed, including for this library, to make CI and advisory checks reproducible.
