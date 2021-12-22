# RINEX 
Rust package to parse and analyze Rinex files

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)
[![codecov](https://codecov.io/gh/gwbres/rinex/branch/main/graph/badge.svg)](https://codecov.io/gh/gwbres/rinex)

This package is **Work in Progress**, its main objective being
able to parse .g and .o observations file, hopefully V3 RINEX standard.

### Getting started

```rust
let rinex = Rinex::from_file("data/amel0010.21g");
println!("{:#?}", rinex);

let rinex = Rinex::from_file("data/aopr0010.17o");
println!("{:#?}", rinex);
```

### Developments

You can add some more RINEX files to the "/data" folder
to test the lib against more data:

```shell
cargo test
cargo test -- --nocapture
```
