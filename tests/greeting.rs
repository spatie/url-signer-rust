//! Public greeting behavior.

use {{crate_import}}::greet;

#[test]
fn greets_the_supplied_name() {
    assert_eq!(greet("Zoë"), "Hello, Zoë!");
    assert_eq!(greet(""), "Hello, !");
}
