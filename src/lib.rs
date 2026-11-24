#![doc = include_str!("../README.md")]

/// Returns a greeting addressed to `name`.
///
/// ```rust
/// # use {{crate_import}}::greet;
/// assert_eq!(greet("Freek"), "Hello, Freek!");
/// ```
#[must_use]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(doctest)]
mod guide {
    include!(concat!(env!("OUT_DIR"), "/documentation.rs"));
}
