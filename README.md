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

| `types::Type`            | Trusted           | Untrusted          | CLI                    | UBX                  |          Notes          |
|--------------------------|-------------------|--------------------|------------------------|----------------------|-------------------------|
| `NavigationData` (NAV)   | V2, V3            |   V4               |  :heavy_check_mark:    | :construction:       |  :heavy_minus_sign:     |
| `ObservationData` (OBS)  | V2, V3            |   V4               |  :heavy_check_mark:    | :construction:       |  :heavy_minus_sign:     |
| `CRINEX` (Compressed OBS)| V1, :sparkles:V3  | :heavy_minus_sign: |  :heavy_check_mark:    | :construction:       | `.XXX.gz` data cannot be understood, <br /> user must manualy uncompress to `.XXX` first |
| `MeteoData` (MET)        | V3, V4            | V2                 |  :heavy_check_mark:    | :heavy_minus_sign:   |  :heavy_minus_sign:     |  
| `ClocksData` (CLK)       | V3                | V4                 |  :construction:        | :heavy_minus_sign:   |  :heavy_minus_sign:     |
| `AntennaData` (ATX)      | V3                | V4                 |  :construction:        | :heavy_minus_sign:   |  :heavy_minus_sign:     |
| `SINEX` (SNX)            | V1                | :heavy_minus_sign: |  :construction:        | :heavy_minus_sign:   |  `SINEX` are special `RINEX`, they are managed by a dedicated <br /> [`core library`](sinex/) |

**CLI** : means exposed to [`rinex-cli`](rinex-cli/) for easy parsing & quick analysis  
**UBX** : means exposed to [`ublox-rnx`](ublox-rnx/) for to produce data with a UBLOX receiver  
:sparkles: `CRINEX` V2 and V4 do not exist  
:heavy_check_mark: supported   
:heavy_minus_sign: not applicable   
:construction: under development  

Note on `V4`: 
- I mark it `untrusted` because it is not under CI/CD (lack of data)
- but parser has been written 
- and format is easy and standardized enough between `types::Type` that there's a good chance it might work.  
:arrow_right_hook: Data, tests, contributions are welcomed

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

* `--with-serde` to enable `Serialization` and `Deserialization`,
useful for applications that need to parse / control some of the
RINEX attributes.
