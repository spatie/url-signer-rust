//! Public signing contracts beyond the PHP differential corpus.

use spatie_url_signer::{
    Error, Expiration, Sha256, Sha256UrlSigner, SignatureAlgorithm, UrlSigner,
};

#[test]
fn expiration_is_inclusive_and_signing_rejects_the_past() {
    let signer = Sha256UrlSigner::new("randomkey")
        .unwrap()
        .with_clock(|| 100);
    let signed = signer.sign("https://myapp.com", 0).unwrap();
    assert!(signer.clone().with_clock(|| 99).validate(&signed));
    assert!(signer.validate(&signed));
    assert!(!signer.with_clock(|| 101).validate(&signed));
    let signer = Sha256UrlSigner::new("randomkey")
        .unwrap()
        .with_clock(|| 100);
    assert_eq!(
        signer.sign("https://myapp.com", -1),
        Err(Error::InvalidExpiration)
    );
    assert_eq!(
        signer.sign("https://myapp.com", Expiration::Timestamp(99)),
        Err(Error::InvalidExpiration)
    );
    assert_eq!(
        signer.sign("https://myapp.com", i64::MAX),
        Err(Error::InvalidExpiration)
    );
}

#[test]
fn default_keys_are_required_but_overrides_can_be_empty() {
    assert!(matches!(
        Sha256UrlSigner::new(""),
        Err(Error::InvalidSignatureKey)
    ));
    let signer = Sha256UrlSigner::new("secret").unwrap().with_clock(|| 100);
    let signed = signer.sign_with_key("https://myapp.com", 30, "").unwrap();
    assert!(signer.validate_with_key(&signed, ""));
    assert!(!signer.validate(&signed));
    assert!(!format!("{signer:?}").contains("secret"));
}

#[test]
fn custom_algorithms_share_normalization_expiration_and_validation() {
    #[derive(Debug)]
    struct Custom;
    impl SignatureAlgorithm for Custom {
        fn create_signature(&self, url: &str, expiration: &str, key: &str) -> String {
            Sha256.create_signature(url, expiration, &format!("custom:{key}"))
        }
    }
    let signer = UrlSigner::with_algorithm("secret", Custom)
        .unwrap()
        .with_parameter_names("until", "hash")
        .with_clock(|| 100);
    let signed = signer.sign("https://myapp.com?foo=a%20b", 30).unwrap();
    assert!(signed.starts_with("https://myapp.com?foo=a+b&until=130&hash="));
    assert!(signer.validate(&signed));
    assert!(!signer.validate(&signed.replace("a+b", "a+c")));
}
