#![doc = include_str!("../README.md")]

use std::{
    fmt,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use hmac::{Hmac, Mac};
use md5::Digest;
use subtle::ConstantTimeEq;

mod query;

/// Errors returned when constructing a signer or signing a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The default signature key must not be empty.
    #[error("The signature key must not be empty")]
    InvalidSignatureKey,
    /// An expiration is in the past or outside the signed 64-bit timestamp range.
    #[error("The expiration must not be in the past or overflow a Unix timestamp")]
    InvalidExpiration,
}

/// A lifetime in seconds, or an absolute Unix timestamp (in seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expiration {
    /// Seconds from the signer's clock, matching PHP's integer argument.
    Seconds(i64),
    /// An absolute Unix timestamp, matching PHP's DateTimeInterface argument.
    Timestamp(i64),
}

impl From<i64> for Expiration {
    fn from(seconds: i64) -> Self {
        Self::Seconds(seconds)
    }
}

/// Signature generation, corresponding to PHP's `createSignature` extension point.
///
/// Implementations must return the exact text stored in the URL. The signer handles
/// PHP query normalization, expiration checks and constant-time text comparison.
pub trait SignatureAlgorithm {
    /// Signs the normalized URL and the original expiration text with the given key.
    fn create_signature(&self, url: &str, expiration: &str, key: &str) -> String;
}

/// HMAC-SHA256, the recommended algorithm and the PHP package's default signer.
#[derive(Debug, Default, Clone, Copy)]
pub struct Sha256;

impl SignatureAlgorithm for Sha256 {
    fn create_signature(&self, url: &str, expiration: &str, key: &str) -> String {
        // HMAC accepts keys of every length, including PHP's empty per-call override.
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key.as_bytes())
            .unwrap_or_else(|_| unreachable!("HMAC accepts every key length"));
        mac.update(url.as_bytes());
        mac.update(b"::");
        mac.update(expiration.as_bytes());
        hex(&mac.finalize().into_bytes())
    }
}

/// Legacy MD5 concatenation for compatibility with PHP's deprecated signer.
///
/// Do not use this for new links. Use [`Sha256`] instead.
#[derive(Debug, Default, Clone, Copy)]
pub struct Md5;

impl SignatureAlgorithm for Md5 {
    fn create_signature(&self, url: &str, expiration: &str, key: &str) -> String {
        let mut digest = md5::Md5::new();
        digest.update(url.as_bytes());
        digest.update(b"::");
        digest.update(expiration.as_bytes());
        digest.update(b"::");
        digest.update(key.as_bytes());
        hex(&digest.finalize())
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    bytes
        .iter()
        .flat_map(|byte| {
            [
                DIGITS[(byte >> 4) as usize] as char,
                DIGITS[(byte & 15) as usize] as char,
            ]
        })
        .collect()
}

/// A signer with PHP-compatible query handling and an injectable seconds clock.
///
/// URL text is deliberately not normalized by a WHATWG URL parser: changing a
/// hostname, port, path or fragment would change the PHP signature.
#[derive(Clone)]
pub struct UrlSigner<A = Sha256> {
    key: String,
    expires_parameter: String,
    signature_parameter: String,
    algorithm: A,
    clock: Arc<dyn Fn() -> i64 + Send + Sync>,
}

impl<A: fmt::Debug> fmt::Debug for UrlSigner<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UrlSigner")
            .field("key", &"[REDACTED]")
            .field("expires_parameter", &self.expires_parameter)
            .field("signature_parameter", &self.signature_parameter)
            .field("algorithm", &self.algorithm)
            .finish_non_exhaustive()
    }
}

/// The PHP-compatible HMAC-SHA256 signer.
pub type Sha256UrlSigner = UrlSigner<Sha256>;
/// The PHP-compatible legacy MD5 signer. Prefer [`Sha256UrlSigner`] for new links.
pub type Md5UrlSigner = UrlSigner<Md5>;

impl<A: SignatureAlgorithm + Default> UrlSigner<A> {
    /// Creates a signer using `expires` and `signature` query parameters.
    ///
    /// # Errors
    /// Returns [`Error::InvalidSignatureKey`] for an empty default key.
    pub fn new(key: impl Into<String>) -> Result<Self, Error> {
        Self::with_algorithm(key, A::default())
    }
}

impl<A: SignatureAlgorithm> UrlSigner<A> {
    /// Creates a signer with a custom signature algorithm.
    ///
    /// # Errors
    /// Returns [`Error::InvalidSignatureKey`] for an empty default key.
    pub fn with_algorithm(key: impl Into<String>, algorithm: A) -> Result<Self, Error> {
        let key = key.into();
        if key.is_empty() {
            return Err(Error::InvalidSignatureKey);
        }
        Ok(Self {
            key,
            expires_parameter: "expires".into(),
            signature_parameter: "signature".into(),
            algorithm,
            clock: Arc::new(unix_time),
        })
    }

    /// Uses custom expiration and signature query parameter names.
    ///
    /// As in PHP, choose distinct plain names without spaces, dots or brackets;
    /// PHP's query parser transforms such names and can make links unverifiable.
    #[must_use]
    pub fn with_parameter_names(
        mut self,
        expires: impl Into<String>,
        signature: impl Into<String>,
    ) -> Self {
        self.expires_parameter = expires.into();
        self.signature_parameter = signature.into();
        self
    }

    /// Uses a clock returning signed Unix seconds, for deterministic validation.
    #[must_use]
    pub fn with_clock(mut self, clock: impl Fn() -> i64 + Send + Sync + 'static) -> Self {
        self.clock = Arc::new(clock);
        self
    }

    /// Signs a URL for a number of seconds or until an absolute timestamp.
    ///
    /// # Errors
    /// Returns [`Error::InvalidExpiration`] for past or overflowing expirations.
    pub fn sign(&self, url: &str, expiration: impl Into<Expiration>) -> Result<String, Error> {
        self.sign_with_key(url, expiration, &self.key)
    }

    /// Signs with a per-call key override (which may be empty, as in PHP).
    ///
    /// # Errors
    /// Returns [`Error::InvalidExpiration`] for past or overflowing expirations.
    pub fn sign_with_key(
        &self,
        url: &str,
        expiration: impl Into<Expiration>,
        key: &str,
    ) -> Result<String, Error> {
        let now = (self.clock)();
        let timestamp = match expiration.into() {
            Expiration::Seconds(seconds) => {
                now.checked_add(seconds).ok_or(Error::InvalidExpiration)?
            }
            Expiration::Timestamp(timestamp) => timestamp,
        };
        if timestamp < now {
            return Err(Error::InvalidExpiration);
        }
        let expiration = timestamp.to_string();
        let intended =
            query::without_parameters(url, &[&self.expires_parameter, &self.signature_parameter]);
        let signature = self.algorithm.create_signature(&intended, &expiration, key);
        Ok(query::add_parameters(
            &intended,
            &[
                (&self.expires_parameter, &expiration),
                (&self.signature_parameter, &signature),
            ],
        ))
    }

    /// Checks the signature and expiry. A URL is still valid at its expiry second.
    #[must_use]
    pub fn validate(&self, url: &str) -> bool {
        self.validate_with_key(url, &self.key)
    }

    /// Validates with a per-call key override.
    #[must_use]
    pub fn validate_with_key(&self, url: &str, key: &str) -> bool {
        let parameters = query::parameters(url);
        let Some(expiration) = parameters.scalar(&self.expires_parameter) else {
            return false;
        };
        let Some(provided) = parameters.scalar(&self.signature_parameter) else {
            return false;
        };
        let Ok(expiration) = std::str::from_utf8(expiration) else {
            return false;
        };
        let Some(timestamp) = query::integer(expiration) else {
            return false;
        };
        if timestamp < (self.clock)() {
            return false;
        }
        let intended =
            query::without_parameters(url, &[&self.expires_parameter, &self.signature_parameter]);
        let valid = self.algorithm.create_signature(&intended, expiration, key);
        bool::from(valid.as_bytes().ct_eq(provided))
    }
}

fn unix_time() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(error) => {
            -(error.duration().as_secs() as i64) - i64::from(error.duration().subsec_nanos() != 0)
        }
    }
}

#[cfg(doctest)]
mod guide {
    include!(concat!(env!("OUT_DIR"), "/documentation.rs"));
}
