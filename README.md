RINEX 
=====

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 


Rust tool suites to parse, analyze, manipulate `RINEX` files

* [rinex](rinex/README.md) is the library /crate, 
it contains the parser and other objects. 

* [crx2rnx](crx2rnx/README.md) is a command line application to decompress a `CRINEX` file

* [rinex-cli](rinex_cli/README.md) is a command line application
to analyze data and perform operations (like `teqc`) on `RINEX` files

* `rnx2crx`: `RINEX` file compressor is currently under development,
see `develop` branches

## Supported RINEX revisions

* 1.00 ⩽ v < 4.0    Tested 
*             v = 4.0    refer to file type specific pages

## Supported `RINEX` files

The following RINEX files are currently supported:

* `Type::NavigationData` (NAV) data
* `Type::ObservationData` (OBS) data
* `Type::MeteoData` (Meteo) data

## CRINEX special case

CRINEX V1 and V3 are fully supported.   
CRINEX V2 does not exist.  

You can directly pass to the parser Observation `RINEX` files that were compressed with 
the `RNX2CRX` official tool.
