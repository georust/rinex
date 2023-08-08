# RINEX

[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![rustc](https://img.shields.io/badge/rustc-1.61%2B-blue.svg)](https://img.shields.io/badge/rustc-1.61%2B-blue.svg)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)

*RINEX* is a crate in the *GeoRust* ecosystem that aims at supporting
most common RINEX formats, for both data analysis and data production,
without performance compromises.

One of its objectives is to be a credible modern alternative to tools like `teqc`.

## File naming conventions

In this current form, the parser disregards file names and conventions. 
Although we aim at providing methods that help generate files that respect the standards,
in file production context.

## Crate features

One crate feature per supported RINEX format exists.   
For example, `nav` enables RINEX Navigation specific methods.

The `qc` feature enables a set of structures for RINEX file quality analysis.  

The  `processing` feature enables the 
[Preprocessing trait](https://docs.rs/rinex/latest/rinex/processing/trait.Preprocessing.html),
to resample, filter and sort RINEX datasets prior further analysis.

The `flate2` feature enables native gz decompression.  
If this feature is not enabled, one must first uncompress .gz files prior parsing.

The `sbas` feature enables one method to select appropriate augmentation system
based on current location on Earth.

## License

Licensed under either 

* Apache Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
* MIT ([LICENSE-MIT](http://opensource.org/licenses/MIT)
