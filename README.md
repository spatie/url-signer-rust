# {{crate_name}}

[![Latest version on crates.io](https://img.shields.io/crates/v/{{crate_name}}.svg?style=flat-square)](https://crates.io/crates/{{crate_name}})
[![Tests](https://img.shields.io/github/actions/workflow/status/{{owner}}/{{repository}}/ci.yml?branch=main&style=flat-square)](https://github.com/{{owner}}/{{repository}}/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://github.com/{{owner}}/{{repository}}/blob/main/LICENSE.md)
[![Total downloads](https://img.shields.io/crates/d/{{crate_name}}.svg?style=flat-square)](https://crates.io/crates/{{crate_name}})

<!--delete-->
---
Use this template to create a Rust package:

1. Press the "Use this template" button to create a new repository with this skeleton's contents.
2. Run `python3 configure.py` (Windows: `py -3 configure.py`) to replace the placeholders and remove the template-only files.
3. Have fun creating your package.

Requires Python 3.11+ for configuration and [Rust](https://rustup.rs/) 1.89+ for the library. See [SKELETON.md](SKELETON.md) for template maintenance and non-interactive use.

---
<!--/delete-->
{{description}}

## Support us

[<img src="https://spatie.be/github-ad-image/{{repository}}.jpg" alt="Support Spatie's open source work" width="419px" />](https://spatie.be/github-ad-click/{{repository}})

We invest a lot of resources into creating [best in class open source packages](https://spatie.be/open-source). You can support us by [buying one of our paid products](https://spatie.be/open-source/support-us).

We highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using. You'll find our address on [our contact page](https://spatie.be/about-us). We publish all received postcards on [our virtual postcard wall](https://spatie.be/open-source/postcards).

## Installation

Requires Rust 1.89 or newer. Once the crate is published:

```sh
cargo add {{crate_name}}
```

Before publication, use a path dependency from a neighboring application:

```toml
[dependencies]
{{crate_name}} = { path = "../{{repository}}" }
```

## Usage

```rust
use {{crate_import}}::greet;

assert_eq!(greet("Freek"), "Hello, Freek!");
```

See the [usage guide](https://github.com/{{owner}}/{{repository}}/blob/main/docs/usage.md).

## Testing

```sh
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
```

See [all quality checks](https://github.com/{{owner}}/{{repository}}/blob/main/docs/testing.md). The README and Markdown guides are tested by rustdoc, including setup on hidden `# ` lines.

## Changelog

See [CHANGELOG.md](https://github.com/{{owner}}/{{repository}}/blob/main/CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](https://github.com/{{owner}}/{{repository}}/blob/main/CONTRIBUTING.md) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

Using an AI coding agent? Load Apollo's [rust-best-practices skill](https://github.com/apollographql/skills/tree/main/skills/rust-best-practices) when writing or reviewing code in this package.

## Security Vulnerabilities

Report vulnerabilities privately using the instructions in [SECURITY.md](https://github.com/{{owner}}/{{repository}}/blob/main/SECURITY.md).

## Credits

- [{{author_name}}](https://github.com/{{author_username}})
- [All contributors](https://github.com/{{owner}}/{{repository}}/graphs/contributors)

## License

The MIT License (MIT). See [LICENSE.md](https://github.com/{{owner}}/{{repository}}/blob/main/LICENSE.md).
