---
title: Introduction
weight: 1
---

This package can create URLs with a limited lifetime. This is done by adding an expiration date and a signature to the URL.

```rust
# fn main() -> Result<(), spatie_url_signer::Error> {
use spatie_url_signer::Sha256UrlSigner;

let url_signer = Sha256UrlSigner::new("randomkey")?;

let signed_url = url_signer.sign("https://myapp.com", 30)?;

// The generated URL will be valid for 30 seconds.
# assert!(url_signer.validate(&signed_url));
# Ok(())
# }
```

This will output a URL that looks like `https://myapp.com?expires=xxxx&signature=xxxx`.

Imagine mailing this URL out to the users of your application. When a user clicks on a signed URL your application can validate it with:

```rust
# use spatie_url_signer::Sha256UrlSigner;
# fn main() -> Result<(), spatie_url_signer::Error> {
# let url_signer = Sha256UrlSigner::new("randomkey")?;
# let signed_url = url_signer.sign("https://myapp.com", 30)?;
// Returns true if valid, false if not.
assert!(url_signer.validate(&signed_url));
# Ok(())
# }
```

The signatures and query formatting match the [PHP package](https://github.com/spatie/url-signer). You can sign in PHP and validate in Rust, or sign in Rust and validate in PHP. See [usage](usage.md) for normalization rules and the compatibility tests.

Continue with [installation](installation-and-setup.md).
