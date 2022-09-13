RINEX 
=====

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)
[![rustc](https://img.shields.io/badge/rustc-1.61%2B-blue.svg)](https://img.shields.io/badge/rustc-1.61%2B-blue.svg)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 


Rust tool suites to parse, analyze, manipulate `RINEX` files

* [`rinex`](rinex/) files parser, this is the core library
* [`crx2rnx`](crx2rnx/) is a command line application to decompress CRINEX files
* [`sinex`](sinex/) files parser, core library

* [`rinex-cli`](rinex-cli/) is a command line application that intends to expose
all the core libraries capacities to the end user, in an easy-to-use and efficient fashion.
It can be used to analyze files or perform some of the `teqc` operations, RINEX post processing
with this tool is currently under development.

* [`ublox-rnx`](ublox-rnx) is an application (CLI) that connects to a `Ublox`
receiver and generates RINEX data quickly & easily.
It is made possible by combining the [ublox](https://github.com/lkolbly/ublox) crate
and the [rinex](rinex/) crate.

* `rnx2crx`: RINEX file compressor is currently under development

## Supported `RINEX` types

| `types::Type`            | Support           | CLI                 | UBX                  | Production        |          Notes          |
|--------------------------|-------------------|---------------------|----------------------|-------------------|-------------------------
| `NavigationData` (NAV)   | :heavy_check_mark:|  :heavy_check_mark: | :construction:       |:construction:     | `epoch` iteration |
| `ObservationData` (OBS)  | :heavy_check_mark:|  :heavy_check_mark: | :construction:       | :construction:    | `epoch` iteration |
| `CRINEX` (Compressed OBS)| :heavy_check_mark:|  :heavy_check_mark: | :construction:       | :construction:    | `epoch` iteration |
| `MeteoData` (MET)        | :heavy_check_mark:| :heavy_check_mark:  | :construction:       |:heavy_check_mark: | `epoch` iteration |  
| `ClocksData` (CLK)       | :heavy_check_mark:|  :heavy_check_mark: | :question:        |:construction: | `epoch` iteration |
| `AntennaData` (ATX)      | :heavy_check_mark:| :sparkle:           | :heavy_minus_sign:   |:construction: | `ATX` records are not indexed by `epochs` |
| `IonosphereMaps` (IONEX) | :sparkle:         |  :sparkle:          | :question:           |:construction: | `epoch` iteration |
| `SINEX` (SNX)            | :construction:    |  :construction:     | :heavy_minus_sign:   |:construction: |   `SINEX` are special `RINEX`, they are managed by a dedicated [core library](sinex/)  |
| `Troposphere` (TRO)      | :construction:    |  :construction:     | :question:           |:construction: |   `Troposphere` are one possible SINEX declination |
| `Bias` (BIA)             | :heavy_check_mark: |  :construction:        | :question:           |:construction: |   `Bias` solutions are one possible SINEX declination |

**Production** means file generation (_to_file()_)    
**CLI** means exposed to [`rinex-cli`](rinex-cli/) for easy parsing & quick analysis  
**UBX** means exposed to [`ublox-rnx`](ublox-rnx/) for quick and easy data production from a UBLOX receiver  

:heavy_check_mark: all revisions supported   
:heavy_minus_sign: not applicable   
:sparkle: parser will work, not fully stabilized: don't expect something extraordinary   
:construction: under development: parser will fail

## Supported file format / compressions

| Format   | File name restrictions  |    Support          |
|----------|-------------------------|---------------------|
| CRINEX   | :heavy_minus_sign: | :heavy_check_mark:  | 
| Others   | :heavy_minus_sign: | Refer to first table |
| CRINEX + `gzip` | Must end with `.gz` | Compile with `--flate2` feature, or uncompress yourself |
| Others + `gzip` | Must end with `.gz` | Refer to first table and compile with `--flate2` feature, or uncompress yourself |
| CRINEX + `zlib` | Must end with `.Z` | :construction:  |
| Others + `zlib` | Must end with `.Z` | :construction:  |

:heavy_minus_sign: no restrictions. We can parse a  CRINEX or a IONEX named foo.txt as long as it follows the standards.      
:heavy_check_mark: natively supported   
:construction:, under development  

## Record (high level) operations

High level operation can be performed using the `Rinex` structure,
or through the command line interface. Refer either

- to the [API](https://docs.rs/rinex/0.6.0/rinex/struct.Rinex.html) documentation
- to the [command-line interface](rinex-cli/README.md) documentation

## Features

* `--serde` enables main RINEX structures serialization and deserialization 

<img align="right" width="400" src="https://upload.wikimedia.org/wikipedia/commons/4/46/SBAS_Service_Areas.png">

* `--with-geo`   
unlocks the 
`augmentation::sbas_selection_helper()` method,
to select the most appropriate `SBAS` augmentation system for
a given (usually current..) location on Earth.
See [constellation](doc/constellation.md) for example of use.

* `--flate2`  
allow native parsing of .gz compressed RINEX files. Otherwise, user must uncompress manually the `.gz` extension first.

## Contributions

Contributions are welcomed, do not hesitate to open new issues. 
If you want to take part in active developments, checkout our [TODO list](TODO.md)
