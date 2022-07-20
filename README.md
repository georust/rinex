RINEX 
=====

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 


Rust tool suites to parse, analyze, manipulate `RINEX` files

* [`rinex`](rinex/) `RINEX` file parsing & production and `teqc` similar ops, 
this is the core library

* [`crx2rnx`](crx2rnx/) is a command line application to decompress `CRINEX` files

* [`sinex`](sinex/) `SINEX` files parser, core library

* [`rinex-cli`](rinex-cli/) is a command line application
to analyze data and perform operations (like `teqc`) on `RINEX`, `CRINEX` 
and soon `SINEX` files

* [`ublox-rnx`](ublox-rnx) is an application (CLI) that connects to a `Ublox`
receiver and generates RINEX data quickly & easily.
It is made possible by combining the [ublox](https://github.com/lkolbly/ublox) crate
and [`rinex`](rinex/) library core.

* `rnx2crx`: `RINEX` file compressor is currently under development,
see `develop` branches

## Supported `RINEX` files

| `types::Type`            | Trusted           | Untrusted          | CLI                    | UBX                  | Production    |          Notes          |
|--------------------------|-------------------|--------------------|------------------------|----------------------|---------------|-------------------------
| `NavigationData` (NAV)   | V2, V3            |   V4               |  :heavy_check_mark:    | :construction:       |:construction: |                         |
| `ObservationData` (OBS)  | V2, V3            |   V4               |  :heavy_check_mark:    | :construction:       |:construction: |                          |
| `CRINEX` (Compressed OBS)| V1, :sparkles:V3  |                    |  :heavy_check_mark:    | :construction:       |:construction: |  `.XXX.gz` data cannot be understood, user must manualy <br /> uncompress to `.XXX` first |
| `MeteoData` (MET)        | V2, V3            |   V4               |  :heavy_check_mark:    | :heavy_minus_sign:   |:construction: |                          |  
| `ClocksData` (CLK)       | V3                |   V4               |  :construction:        | :question:           |:construction: |                          |
| `AntennaData` (ATX)      | :construction:    |                    |  :construction:        | :heavy_minus_sign:   |:construction: |                          |
| `IonosphereMaps` (Iono)  | :construction:    |                    |  :construction:        | :question:           |:construction: |                          |
| `SINEX` (SNX)            | :construction:    |                    |  :construction:        | :heavy_minus_sign:   |:construction: |   `SINEX` are special `RINEX`, they are managed by a dedicated <br /> [`core library`](sinex/) |
| `Troposphere` (TRO)      | :construction:    |                    |  :construction:        | :question:           |:construction: |   `Troposphere` are one possible declination of SINEX files |
| `Bias` (BIA)             | :construction:    |                    |  :construction:        | :question:           |:construction: |   `Bias` solutions are one possible declination of SINEX files |

`trusted`: means under CI/CD, user can parse safely   
`untrusted`: means not under CI/CD, either due to lack of test data, partial (:construction:) or incomplete support   

Notes on `V4`: 
- always marked as `untrusted` to this day, due to lack of data
- there's a good chance OBS/NAV/MET will work, because format is actually simpler
and parser has been coded.
:arrow_right_hook: Data, tests and contributions are welcomed

**Production** means file generation (_to_file()_) of `trusted` revisions  
**CLI** means exposed to [`rinex-cli`](rinex-cli/) for easy parsing & quick analysis  
**UBX** means exposed to [`ublox-rnx`](ublox-rnx/) for to produce data with a UBLOX receiver  
:sparkles: `CRINEX` V2 and V4 do not exist  
:heavy_check_mark: supported   
:heavy_minus_sign: not applicable   
:construction: under development  


## `teqc` special operations

| Ops      | Status          | 
|----------|-----------------|
| `Merge` | :construction:   |
| `Splice` | :construction:  | 

## Custom special operations

| Ops           | Status          | 
|---------------|-----------------|
| `Down sample` | :construction:  |

## Features

* `--with-serde`   
enables `Serialization` and `Deserialization` of key RINEX structures

<img align="right" width="400" src="https://upload.wikimedia.org/wikipedia/commons/4/46/SBAS_Service_Areas.png">

* `--with-geo`   
will be provided in 0.4.0. The feature
includes the `rust::geo` crate, 
and unlocks the    
`augmentation::sbas_selection_helper()` method,
to select the most appropriate `SBAS` augmentation system for
a given (usually current..) location on Earth.

* `--with-gzip`  
to be provided in future, allows parsing .gz compressed RINEX files directly

## Contributions

Contributions, raw data and tests methods are welcomed.  
There is still a lot to achieve with this lib, especially regarding the command line applications (high level usage of the library cores).

### Introducing new RINEX types

Follow the existing architecture:

* introduce `types::Type::foo`
* provide new `record::Record` declination
* create `rinex/src/foo` sub directory and provide at least a rinex/src/foo/record.rs for the file body
* add new specific header fields if need be, define them in `rinex/src/foo`
* attach unit tests to the new `rinex/src/foo` structures & methods
* provide relevant (but truncated, to keep repo size reasonnable) raw data, under `test_resources/`
* add new type to `test_resources` testbench in `tests/parser.rs`
* add a focused testbench, in `tests/foo.rs` with specific fields test

### Adding more RINEX data

* only introduce non existing RINEX declinations
* truncate huge files to maintain a reasonnable repo size 
