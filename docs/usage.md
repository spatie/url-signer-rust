---
title: Usage
weight: 4
---

Pass a name to `greet`:

```rust
# use {{crate_import}}::greet;
# let name = "Freek";
let greeting = greet(name);

assert_eq!(greeting, "Hello, Freek!");
```

Names are UTF-8 strings. An empty name produces an empty space before the exclamation mark:

```rust
# use {{crate_import}}::greet;
assert_eq!(greet("Zoë"), "Hello, Zoë!");
assert_eq!(greet(""), "Hello, !");
```
