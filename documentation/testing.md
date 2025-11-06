
# Testing

## Coverage

Unit test coverage is reported using the crate
[`cargo-tarpaulin`](https://crates.io/crates/cargo-tarpaulin).
This tool is not entirely accurate so even though it reports test coverage below
100%, all of the lines that are reportedly skipped cannot be as lines before
them are not skipped. This can be more clearly seen by generating the **html**
output by running `cargo tarpaulin --out Html`. Running just the tests without
code coverage can be done by running `cargo test`.

## Methodology

