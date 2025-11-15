
# Testing

## Coverage

Unit test coverage is reported using the crate
[`cargo-tarpaulin`](https://crates.io/crates/cargo-tarpaulin).
This tool is not entirely accurate so even though it reports test coverage below
100%, some of the lines that are reportedly skipped cannot be as lines before
them are not skipped. This can be more clearly seen by generating the **html**
output by running `cargo tarpaulin --out Html`. Running just the tests without
code coverage can be done by running `cargo test`.

Unit tests which test the functionality of a given module live in the same file
as that module. They are in the submodule `test` which is marked by
`#[cfg(test)]`.

As of the last commit the coverage is: 95.09%.

## Methodology

The name of the test functions give hints as to what specific thing I am
testing. In addition, I have placed some comments inside the tests to provide
further insight as to how and why I am testing a given procedure.

TODO describe the tests in the following files:
a_star.rs
binary_heap.rs
grid.rs
mst.rs
room.rs
triangulation.rs
vec.rs

