RINEX 
=====

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)

[![minimum rustc: 1.64](https://img.shields.io/badge/minimum%20rustc-1.64-blue?logo=rust)](https://www.whatrustisit.com)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 

Rust tool suites to parse, analyze and process [RINEX Data](https://en.wikipedia.org/wiki/RINEX).

* [`rinex`](rinex/) is the core library 
* [`rinex-cli`](rinex-cli/) is a command line application based on the core library.  
It can be used to process RINEX files and perform operations similar to `teqc`.   
The application is auto-generated for a few architectures, download it from the 
[release portal](https://github.com/gwbres/rinex/releases)

* [`rnx2crx`](rnx2crx/) is a RINEX compression program 
* [`crx2rnx`](crx2rnx/) is a CRINEX decompression program (Compact RINEX to RINEX)
* [`sinex`](sinex/) SNX dedicated core library

* [`ublox-rnx`](ublox-rnx/) is an application that connects to a `Ublox`
receiver and generates RINEX data quickly & easily.   
It is the combination of the [ublox](https://github.com/lkolbly/ublox)
and [rinex](rinex/) crates.

By default, all timestamps are in UTC with leap seconds correctly managed.

This RINEX toolsuite is part of the [GeoRust community](https://github.com/georust),
and we aim towards advanced geodesic and ionospheric analysis.

RINEX Standards
===============

| Type                       | Parser            | Writer              |  CLI                 | UBX                  |           Notes          |
|----------------------------|-------------------|---------------------|----------------------|-------------------|-------------------------
| Navigation  (NAV)          | :heavy_check_mark:| Ephemeris :construction: V4 :construction: |  :heavy_check_mark: :chart_with_upwards_trend:  | :construction:       | Epoch iteration |
| Observation (OBS)          | :heavy_check_mark:| :heavy_check_mark: | :heavy_check_mark:  :chart_with_upwards_trend: |  :construction:  | Epoch iteration |
|  CRINEX  (Compressed OBS)  | :heavy_check_mark:| RNX2CRX1 :heavy_check_mark: RNX2CRX3 :construction:  | :heavy_check_mark:  :chart_with_upwards_trend:  |  :construction:    | Epoch iteration |
|  Meteorological data (MET) | :heavy_check_mark:| :heavy_check_mark:  | :heavy_check_mark: :chart_with_upwards_trend:  | :construction:  | Epoch iteration |  
|  Clocks (CLK)              | :heavy_check_mark:| :construction:      | :construction:   |:construction: | Epoch iteration |
|  Antenna (ATX)             | :heavy_check_mark:| :construction:      | :construction:   |:construction: | Sorted by `antex::Antenna` |
|  Ionosphere Maps  (IONEX)  | :heavy_check_mark:|  :construction:     | :heavy_check_mark:  :chart_with_upwards_trend: |:construction: | Epoch iteration |
|  SINEX  (SNX)              | :construction:    |  :construction:     | :heavy_minus_sign:   |:construction: | SINEX are special RINEX, they are managed by a dedicated [core library](sinex/)  |
|  Troposphere  (TRO)        | :construction:    |  :construction:     | :question:           |:construction: | Troposphere are one possible SINEX declination |
|  Bias  (BIA)               | :heavy_check_mark: |  :construction:    | :question:           |:construction: | Bias solutions are one possible SINEX declination |

:heavy_check_mark: means all revisions supported   
:construction: under development   
__CLI__ + :chart_with_upwards_trend: means record analysis is supported by the CLI, [README](rinex-cli/README.md)

File formats
============

| Format                 | File name restrictions            |    Support                         |
|------------------------|-----------------------------------|------------------------------------|
| RINEX                  | :heavy_minus_sign:                | :heavy_check_mark:                 |
| CRINEX                 | :heavy_minus_sign:                | :heavy_check_mark:                 | 
| gzip compressed RINEX  | Name must end with `.gz`          | `--flate2` feature must be enabled |
| gzip compressed CRINEX | Name must end with `.gz`          | `--flate2` feature must be enabled |

:heavy_minus_sign: No restrictions: file names do not have to follow naming conventions.  

## Known weaknesses :warning:

- Glonass Time Scale is not known to this day.
We cannot parse and apply system time corrections from other time scales into the glonass time scale.

Benchmarking
============

Test           | Results 
---------------|-------------------------|
textdiff/decompression/epoch | 979.55 ns |
textdiff/decompression/flag  | 147.16 ns | 
numdiff/decompression/small  | 191.86 ns |
numdiff/decompression/big    | 1.0973 µs |
parsing/OBSv2/zegv0010.21o   | 951.40 µs |
parsing/OBSv3/ACOR00ESP      | 4.1139 ms |
processing/esbc00dnkr2021/mask:gnss | 352.81 ms |
processing/esbc00dnkr2021/mask:obs |  438.73 ms |
processing/esbc00dnkr2021/mask:sv | 341.42 ms | 
processing/esbc00dnkr2021/smooth:hatch:l1c,l2c | 502.90 ms | 

Contributions
=============

Contributions are welcomed, do not hesitate to open new issues
and submit Pull Requests through Github.

If you want to take part in active developments, check out our [contribution guidelines and hints](CONTRIBUTING.md) to navigate this library quicker.
