RINEX 
=====

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)

[![minimum rustc: 1.61](https://img.shields.io/badge/minimum%20rustc-1.61-blue?logo=rust)](https://www.whatrustisit.com)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 


Rust tool suites to parse, analyze, manipulate `RINEX` files

* [`rinex`](rinex/) is the core library 
* [`rnx2crx`](rnx2crx/) is a RINEX compression program 
* [`crx2rnx`](crx2rnx/) is a CRINEX decompression program (Compact RINEX to RINEX)
* [`sinex`](sinex/) SNX dedicated core library

* [`rinex-cli`](rinex-cli/) is a command line application to process RINEX data
and make use of this library. It can perform some `teqc` operations and perform
Differential analysis.

* [`ublox-rnx`](ublox-rnx) is an application (CLI) that connects to a `Ublox`
receiver and generates RINEX data quickly & easily.
It is made possible by combining the [ublox](https://github.com/lkolbly/ublox) crate
and the [rinex](rinex/) crate.

## Supported `RINEX` types

| Type                       | Parser            | Writer              |  CLI                 | UBX                  |           Notes          |
|----------------------------|-------------------|---------------------|----------------------|-------------------|-------------------------
| Navigation  (NAV)          | :heavy_check_mark:| :construction: |  :heavy_check_mark:  | :construction:       | Epoch iteration |
| Observation (OBS)          | :heavy_check_mark:| :heavy_check_mark: | :heavy_check_mark:  :chart_with_upwards_trend: |  :construction:  | Epoch iteration |
|  CRINEX  (Compressed OBS)  | :heavy_check_mark:| :construction:  | :heavy_check_mark:  :chart_with_upwards_trend:  |  :construction:    | Epoch iteration |
|  Meteorological data (MET) | :heavy_check_mark:| :heavy_check_mark:  | :heavy_check_mark: :chart_with_upwards_trend:  | :construction:  | Epoch iteration |  
|  Clocks (CLK)              | :heavy_check_mark:| :construction:          | :question:           |:construction: | Epoch iteration |
|  Antenna (ATX)             | :heavy_check_mark:| :construction:      | :heavy_minus_sign:   |:construction: | Sorted by `antex::Antenna` |
|  Ionosphere Maps  (IONEX)  | :construction:         |  :construction:     | :question:           |:construction: | Epoch iteration |
|  SINEX  (SNX)              | :construction:    |  :construction:     | :heavy_minus_sign:   |:construction: | SINEX are special RINEX, they are managed by a dedicated [core library](sinex/)  |
|  Troposphere  (TRO)        | :construction:    |  :construction:     | :question:           |:construction: | Troposphere are one possible SINEX declination |
|  Bias  (BIA)               | :heavy_check_mark: |  :construction:    | :question:           |:construction: | Bias solutions are one possible SINEX declination |

:heavy_check_mark: means all revisions supported   
:construction: under development   
:chart_with_upwards_trend: means graphical RINEX Record analysis is possible, [README](rinex-cli/README.md)

## File formats

| Format   | File name restrictions  |    Support          |
|----------|-------------------------|---------------------|
| RINEX    | :heavy_minus_sign: | :heavy_check_mark: but refer to first table |
| CRINEX   | :heavy_minus_sign: | :heavy_check_mark:  | 
| RINEX + `gzip`   | Must end with `.gz` | Compile with `--flate2` feature, or uncompress manually first |
| CRINEX + `gzip` | Must end with `.gz` | Compile with `--flate2` feature, or uncompress manually first |
| `.Z` | :heavy_minus_sign:  | :x: |

:heavy_minus_sign: No restrictions: file names do not have to follow naming conventions.  

## Record

High level operations can be performed on RINEX records and
RINEX structure in general.
Refer to the [official Documentation](https://docs.rs/rinex/latest/rinex/struct.Rinex.html).

RINEX Records vary a lot from one revision to another
and from one file type to another.
To learn how to browse the RINEX record you are interested in,
refer to its definition in the official documentation.
For example, here is the 
[Observation Record](https://docs.rs/rinex/latest/rinex/observation/record/type.Record.html)
definition.

## Features

* `--serde` enables main RINEX structures serialization and deserialization 

<img align="right" width="400" src="https://upload.wikimedia.org/wikipedia/commons/4/46/SBAS_Service_Areas.png">

* `--with-geo`   
unlocks the 
[sbas_selection_help()](https://docs.rs/rinex/0.7.0/rinex/struct.Rinex.html) method,
to select the most appropriate `SBAS` augmentation system for
a given (usually current..) location on Earth.

* `--flate2`  
allow native parsing of .gz compressed RINEX files. Otherwise, user must uncompress manually the `.gz` extension first.

## Performances

Parsing and `--sv` enumeration requested with `rinex-cli`

File           |  RINEX 0.6 `debug`  | RINEX 0.7 `debug` | RINEX 0.7 `--release`        |
---------------|---------------------|-------------------|------------------------------|
ESBC00DNK      |  26s                | 14s               | 2s                           |
ESBC00DNK.gz   |  26s                | 14s               | 2s                           |
MOJN00DNK      |  28s                | 13s               | 2s                           |
MOJN00DNK.gz   |  28s                | 13s               | 2s                           |

Always compile rust code with the `--release` flag :+1: 

## Contributions

Contributions are welcomed, do not hesitate to open new issues
and submit Pull Requests.

We're looking for Ionosphere Maps (IONEX) to put our parser to the test, providing such data would be really appreciated.

If you want to take part in active developments, checkout our [TODO list](TODO.md)
