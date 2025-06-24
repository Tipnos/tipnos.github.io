+++
title = 'My first open source rust crate: explicit-error'
date = 2025-06-19T08:27:25+02:00
tags = ['rust', 'error handling', 'programming pattern']
description = 'Provide tools to have an explicit and concise error syntax for binary crates.'
[params]
    enableComments = true
+++

While working on the new Outscale's [IAM](https://en.wikipedia.org/wiki/Identity_and_access_management), the team encountered challenges with errors handling in Rust. At the beginning we naively used the [thiserror](https://crates.io/crates/thiserror) crate because members had good experiences using it in open source libraries and HTTP servers. It was so simple to implement and use that we never reconsidered our choice until the consequences painfully arose. This over simplicity warned us, but we didn't pay attention. I was in charge of tackling the issue and ended up publishing my first rust open source crate: [explicit-error](https://crates.io/crates/explicit-error). We refactored the IAM major rust binaries with it and acknowledged an important improvement.

This article describes the crate publishing journey with the following plan:
1. Explain the issues we encountered with thiserror
2. Why other error handling crates like [anyhow](https://crates.io/crates/anyhow) do not fully fill the bill
3. A tour of the explicit-error crate
4. The noticed improvements

# Issues with thiserror

[Google comprehensive rust](https://google.github.io/comprehensive-rust/error-handling/thiserror.html) describes the crate as: 'The thiserror crate provides macros to help avoid boilerplate when defining error types. It provides derive macros that assist in implementing `From<T>`, `Display`, and the `Error` trait'. Combined with enums, error conversions are really easy to implement and aggregate. The `?` operator can be used everywhere without requiring inline conversion with `map_err`. It is where the main issue comes from. Because it's so simple, implicit error conversions are used everywhere with the following consequences: 
- The error handling flow is difficult to understand as it requires to trace back the implicit errors conversion chain. It often ends up jumping from different lines and files.
- To reduce the errors conversion chain length and have consistency in error format and monitoring, errors aggregation in a small number of enums is encourage. Consequently, it increases the coupling and decreases the cohesion. Something every programmers try to avoid!
- When the application reach a mature size, developers can use the `?` operator in lots of places without adding new conversions. They forget to think about the final returned errors which leads to bugs.

To successfully illustrate the issue and its consequences, the example below contains all the layers of an HTTP server. Also some implementations might look rough and not realistic. It is to simply demonstrate patterns that have been encountered.

## Example



```rust

```

# Why not anyhow?

# The explicit-error crate



