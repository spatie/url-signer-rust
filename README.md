# spatie-url-signer

[![Latest version on crates.io](https://img.shields.io/crates/v/spatie-url-signer.svg?style=flat-square)](https://crates.io/crates/spatie-url-signer)
[![Tests](https://img.shields.io/github/actions/workflow/status/spatie/url-signer-rust/ci.yml?branch=main&style=flat-square)](https://github.com/spatie/url-signer-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://github.com/spatie/url-signer-rust/blob/main/LICENSE.md)
[![Total downloads](https://img.shields.io/crates/d/spatie-url-signer.svg?style=flat-square)](https://crates.io/crates/spatie-url-signer)

This package can create URLs with a limited lifetime. This is done by adding an expiration date and a signature to the URL. Links signed by [spatie/url-signer](https://github.com/spatie/url-signer) in PHP can be validated in Rust, and vice versa.

## Support us

[<img src="https://spatie.be/github-ad-image/url-signer-rust.jpg" alt="Support Spatie's open source work" width="419px" />](https://spatie.be/github-ad-click/url-signer-rust)

We invest a lot of resources into creating [best in class open source packages](https://spatie.be/open-source). You can support us by [buying one of our paid products](https://spatie.be/open-source/support-us).

We highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using. You'll find our address on [our contact page](https://spatie.be/about-us). We publish all received postcards on [our virtual postcard wall](https://spatie.be/open-source/postcards).

## Installation

Requires Rust 1.89 or newer.

```sh
cargo add spatie-url-signer
```

## Usage

```rust
# fn main() -> Result<(), spatie_url_signer::Error> {
use spatie_url_signer::Sha256UrlSigner;

let url_signer = Sha256UrlSigner::new("randomkey")?;
let signed_url = url_signer.sign("https://myapp.com", 30)?;

// The generated URL will be valid for 30 seconds.
assert!(url_signer.validate(&signed_url));
# Ok(())
# }
```

This will output a URL that looks like `https://myapp.com?expires=xxxx&signature=xxxx`. A URL is still valid at its exact expiration second.

Use `Expiration::Timestamp(unix_seconds)` for an absolute expiration date, `sign_with_key` / `validate_with_key` for a per-call key, and `with_parameter_names` to customize the query names. Custom algorithms implement `SignatureAlgorithm`. The legacy `Md5UrlSigner` is available for existing PHP links; use `Sha256UrlSigner` for new links.

See the [documentation](https://spatie.be/docs/url-signer-rust/v1/introduction) and [usage guide](https://spatie.be/docs/url-signer-rust/v1/usage), including PHP query and fragment behavior. The signer authenticates a URL's contents; it does not encrypt them or prevent reuse before expiry.

## Testing

```sh
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
```

See [all quality checks](https://github.com/spatie/url-signer-rust/blob/main/docs/testing.md). The README and Markdown guides are tested by rustdoc, including setup on hidden `# ` lines.

## Changelog

See [CHANGELOG.md](https://github.com/spatie/url-signer-rust/blob/main/CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](https://github.com/spatie/url-signer-rust/blob/main/CONTRIBUTING.md) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

Using an AI coding agent? Load Apollo's [rust-best-practices skill](https://github.com/apollographql/skills/tree/main/skills/rust-best-practices) when writing or reviewing code in this package.

## Security Vulnerabilities

Report vulnerabilities privately using the instructions in [SECURITY.md](https://github.com/spatie/url-signer-rust/blob/main/SECURITY.md).

## Credits

- [Freek Van der Herten](https://github.com/freekmurze)
- [Sebastian De Deyne](https://github.com/sebastiandedeyne)
- [All contributors](https://github.com/spatie/url-signer-rust/graphs/contributors)

## License

The MIT License (MIT). See [LICENSE.md](https://github.com/spatie/url-signer-rust/blob/main/LICENSE.md).
