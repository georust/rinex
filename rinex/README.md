RINEX crate
===========

[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![rustc](https://img.shields.io/badge/rustc-1.61%2B-blue.svg)](https://img.shields.io/badge/rustc-1.61%2B-blue.svg)

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex) 
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT)

## Parser

`RINEX` files contain a lot of data and this library is capable of parsing most of it.   
To fully understand how to operate this library, refer
to the official API, which provides useful information.
You can also refer to the examples of use provided by the library.

## Production (writer)

Once a RINEX structure is parsed, it is possible to dump it into a file,
and possibily rework it in the meantime.

## File naming convention

This parser does not care about the RINEX file name.
That means it is possible to parse a file    
that does not respect standard naming conventions.
