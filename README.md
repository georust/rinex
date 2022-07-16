RINEX 
=====

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 


Rust tool suites to parse, analyze, manipulate `RINEX` files

* [rinex](rinex/) is the library /crate, 
it contains the parser and other objects. 

* [crx2rnx](crx2rnx/) is a command line application to decompress a `CRINEX` file

* [rinex-cli](rinex-cli/) is a command line application
to analyze data and perform operations (like `teqc`) on `RINEX` files

* [sinex](sinex/) SINEX special files parsing

* [ublox-rnx](ublox-rnx) is an application that connects to a `Ublox`
receiver and generates Observation and Navigation Data easily.
It is based on the [ublox](https://github.com/lkolbly/ublox) crate.

* `rnx2crx`: `RINEX` file compressor is currently under development,
see `develop` branches

## Supported RINEX revisions

* 1.00 ⩽ v < 4.0    Tested 
*             v = 4.0    refer to file type specific pages

## Supported `RINEX` files

The following RINEX files are currently supported:

* `Type::NavigationData` (NAV) data
* `Type::ObservationData` (OBS) data
* `Type::MeteoData` (MET) data
* `Type::ClockData`: Clocks RINEX

## CRINEX special case

CRINEX V1 and V3 are fully supported.   
CRINEX V2 does not exist.  

You can directly pass to the parser Observation `RINEX` files that were compressed with 
the `RNX2CRX` official tool.

## Features

* `--with-serde` to enable `Serialization` and `Deserialization`,
useful for applications that need to parse / control some of the
RINEX attributes. 

## Coming in next releases

* Improved Clocks RINEX support
* Merge / Splice / Split record special operations
* Antex (ATX) RINEX parsing
