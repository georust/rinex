RINEX 
=====

[![Rust](https://github.com/georust/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/rust.yml)
[![Rust](https://github.com/georust/rinex/actions/workflows/daily.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/daily.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)

[![minimum rustc: 1.64](https://img.shields.io/badge/minimum%20rustc-1.64-blue?logo=rust)](https://www.whatrustisit.com)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/georust/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/georust/rinex/blob/main/LICENSE-MIT) 

Parser and write for [RINEX](https://en.wikipedia.org/wiki/RINEX), SP3 and SINEX data.
All these formats are open source solutions to answer the requirements of GNSS navigation.

Use [Github Issues](https://github.com/georust/rinex/issues) to report bugs and other malfunctions.  
You can also open a [Discussion](https://github.com/georust/rinex/discussions) or leave us a message [on Discord](https://discord.gg/Fp2aape).

## Advantages :rocket: 

- Fast
- Open sources: read and access all the code!
- All modern GNSS constellations, codes and signals
  - Surveying with GPS, Galileo, BeiDou and QZSS
- Time scales: GPST, QZSST, BDT, GST, UTC, TAI
- RINEX V4 full support
- Efficient seamless compression and decompression
- Most RINEX formats supported (see following table)
- High Precision Clock RINEX products (for PPP)
- High Precision Orbital [SP3 for PPP](https://docs.rs/sp3/1.0.7/sp3/)
- DORIS (special RINEX)
- Many pre-processing algorithms including Filter Designer
- Several file operations: merging, splitting, time binning (batch)

## Warnings :warning:

- The BINEX parser does not support all frames yet
- Navigation is currently not feasible with Glonass and IRNSS
- Differential navigation (SBAS, DGNSS or RTK) is not support yet
- Our applications do not accept proprietary formats like Septentrio for example
- File production might lack some features, mostly because we're currently focused on data processing

## Repository 

* [`rinex`](rinex/) is the core library 
* [`tutorials`](tutorials/) is a superset of scripts (Linux/MacOS compatible)
to get started quickly. The examples span pretty much everything our applications allow.
* [`sp3`](sp3/) High Precision Orbits (by IGS) 
* [`binex`](binex/) BINEX Encoding and Decoding library
* [`sinex`](sinex/) SNX dedicated core library
* [`tools`](tools/) are utility scripts and development tools

This repository now only hosts parser libraries: previous applications have been moved to the [RTK-rs](https://github.com/rtk-rs) workspace.

## Citation and referencing

If you need to reference this work, please use the following model:

`GeoRust RINEX Team (2023), RINEX: analysis and processing (Apache-2/MIT), https://georust.org`

Formats & revisions
===================

The parser supports RINEX V4.0, that includes RINEX V4 Navigation files.   
We support the latest revisions for both IONEX and Clock RINEX.  
We support the latest (rev D) SP3 format.  

RINEX formats & applications
============================

| Type                       | Parser            | Writer              |  CLI                 |      Content         | Record Iteration     | Timescale  |
|----------------------------|-------------------|---------------------|----------------------|----------------------|----------------------| -----------|
| Navigation  (NAV)          | :heavy_check_mark:| :construction:      |  :heavy_check_mark: :chart_with_upwards_trend:  | Ephemerides, Ionosphere models | Epoch | SV System time broadcasting this message |
| Observation (OBS)          | :heavy_check_mark:| :heavy_check_mark: | :heavy_check_mark:  :chart_with_upwards_trend: | Phase, Pseudo Range, Doppler, SSI | Epoch | GNSS (any) |
|  CRINEX  (Compressed OBS)  | :heavy_check_mark:| RNX2CRX1 :heavy_check_mark: RNX2CRX3 :construction:  | :heavy_check_mark:  :chart_with_upwards_trend:  |  Phase, Pseudo Range, Doppler, SSI | Epoch | GNSS (any) |
|  Meteorological data (MET) | :heavy_check_mark:| :heavy_check_mark:  | :heavy_check_mark: :chart_with_upwards_trend:  | Meteo sensors data (Temperature, Moisture..) | Epoch | UTC | 
|  Clocks (CLK)              | :heavy_check_mark:| :construction:      | :heavy_check_mark: :chart_with_upwards_trend:  | Precise SV and Reference Clock states |  Epoch | GNSS (any) |
|  Antenna (ATX)             | :heavy_check_mark:| :construction:      | :construction:   | Precise RX/SV Antenna calibration | `antex::Antenna` | :heavy_minus_sign: |
|  Ionosphere Maps  (IONEX)  | :heavy_check_mark:|  :construction:     | :heavy_check_mark:  :chart_with_upwards_trend: | Ionosphere Electron density | Epoch | UTC |
|  DORIS RINEX               | :heavy_check_mark:|  :construction:     | :heavy_check_mark:   | Temperature, Moisture, Pseudo Range and Phase observations | Epoch | TAI |
|  SINEX  (SNX)              | :construction:    |  :construction:     | :heavy_minus_sign:   | SINEX are special RINEX, they are managed by a dedicated [core library](sinex/) | Epoch | :question: |
|  Troposphere  (TRO)        | :construction:    |  :construction:     | :question:           | Troposphere modeling | Epoch | :question: |
|  Bias  (BIA)               | :heavy_check_mark: |  :construction:    | :question:           | Bias estimates, like DCB.. | Epoch | :question: |

Other formats
=============

Contributions
=============

Contributions are welcomed, do not hesitate to open new issues
and submit Pull Requests through Github.

If you want to take part in active developments, check out our [contribution guidelines and hints](CONTRIBUTING.md) to navigate this library quicker.
