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
| `MeteoData` (MET)        | V3, V4            | V2                 |  :heavy_check_mark:    | :heavy_minus_sign:   |:construction: |                          |  
| `ClocksData` (CLK)       | V3                | V4                 |  :construction:        | :question:           |:construction: |                          |
| `AntennaData` (ATX)      | :construction:    |                    |  :construction:        | :heavy_minus_sign:   |:construction: |                          |
| `IonosphereMaps` (Iono)  | :construction:    |                    |  :construction:        | :question:           |:construction: |                          |
| `SINEX` (SNX)            | :construction:    |                    |  :construction:        | :heavy_minus_sign:   |:construction: |   `SINEX` are special `RINEX`, they are managed by a dedicated <br /> [`core library`](sinex/) |
| `Troposphere` (TRO)      | :construction:    |                    |  :construction:        | :question:           |:construction: |   `Troposphere` are one possible declination of SINEX files |
| `Bias` (BIA)             | :construction:    |                    |  :construction:        | :question:           |:construction: |   `Bias` solutions are one possible declination of SINEX files |

**Production** means file generation (_to_file()_) of `trusted` revisions  
**CLI** means exposed to [`rinex-cli`](rinex-cli/) for easy parsing & quick analysis  
**UBX** means exposed to [`ublox-rnx`](ublox-rnx/) for to produce data with a UBLOX receiver  
:sparkles: `CRINEX` V2 and V4 do not exist  
:heavy_check_mark: supported   
:heavy_minus_sign: not applicable   
:construction: under development  

Notes on `V4`: 
- marked `untrusted` to this day because it is not under CI/CD (due to lack of data)
- but parser is written and there's a good chance it might work
:arrow_right_hook: Data, tests and contributions are welcomed

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
enables `Serialization` and `Deserialization` of main RINEX structures
