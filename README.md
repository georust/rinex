RINEX 
=====

[![Rust](https://github.com/georust/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/rust.yml)
[![Rust](https://github.com/georust/rinex/actions/workflows/daily.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/daily.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)

[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/georust/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/georust/rinex/blob/main/LICENSE-MIT) 

Rust framework to parse [RINEX](https://en.wikipedia.org/wiki/RINEX), 
SP3 files, and process GNSS data for navigation and much more.

Use the [Github interface](https://github.com/georust/rinex/issues) to report issues.    
Reach out to us on [Discord](https://discord.gg/Fp2aape).  
To keep track of the latest developments, read the [Changelog](https://github.com/georust/rinex/main/CHANGELOG.md)
and keep an eye on the `#rinex` channel on Discord.

## Advantages :rocket: :artificial_satellite:

- Render High level reports from any supported file format
  - including Geodetic survey results
- Integrated PPP solver
- Special CGGTTS solutions solver
- Open sources: read and access all the code!
- Complete set of examples and tutorials within the repo
- All modern GNSS constellations, codes and signals
  - Surveying with GPS, Galileo, BeiDou and QZSS
- Time scales: GPST, QZSST, BDT, GST, UTC, TAI
- Efficient seamless compression and decompression
- RINEX V4 full support
- All RINEX formats supported (see following table), including
 - High Precision Clock products (for PPP)
 - IONEX (Ionosphere TEC maps)
 - DORIS (special ground station evaluation from space)
- [SP3 (High Precision Products)](https://docs.rs/sp3/1.0.7/sp3/) also supported (for PPP)
- Many pre-processing algorithms
- Several file operations: merging, splitting, file batch synthesis..

## Disadvantages :warning:

- Navigation is currently not feasible with Glonass and IRNSS (applications/ ppp solver).
- QZSS navigation has not been tested yet
- PPP solver and Navigation in general using SBAS is not 100 % feasible yet
- RTK navigation is not feasible yet (work in progress) 
- Our applications do not accept proprietary formats like Septentrio for example
- BINEX support is currently work in progress.
Library exists and works, not integrated to applications yet.

## Citation and referencing

If you need to reference this work, please use the following model:

`GeoRust RINEX Team (2024), RINEX: analysis and processing (Apache-2/MIT), https://georust.org`

Getting started
===============

[Follow our tutorials](./tutorials) to get started.   
We recommend starting with basic examples and following the topics you are interested in.

Relevant Ecosystems
===================

* [IGS Network](https://network.igs.org/): browse and monitor any IGS station status
* [Nyx-space](https://github.com/nyx-space/nyx): Navigation and Orbital calculations in Rust
* [Hifitime](https://github.com/nyx-space/hifitime): Precise Time and Timescale support in Rust
* [CGGTTS](https://github.com/gwbres/cggtts): Common View Time Transfer file format, in Rust
* [Geo](https://github.com/georust/geo): Geospatial primitives and algorithms, in Rust
- [RTK-RS](https://github.com/rtk-rs/gnss-rtk): Precise Positioning (calculations) in Rust
* [GNSS](https://github.com/rtk-rs/gnss) in Rust

Repo architecture
=================

This repo holds everything for GNSS post processing.
Including official Rust libraries, some applications,
tutorials on how to use the applications and data sets,
mostly for testing and demonstration purposes.

### Applications

* [`BIN2RNX`](bin2rnx/) is an application to collect a BINEX stream into RINEX files.
* [`CRX2RNX`](crx2rnx/) is a CRINEX decompresor (Compact RINEX to RINEX).
It is a light application that you can combined to `rinex-cli` for a complete workflow.
* [`RINEX-Cli`](rinex-cli/) is our main application.
It is a Cli and is not a GUI. A GUI will be developped once
all most vital post processing has been achieved.
If you want to see this happen sooner, contact us either on Discord or Github.com
and help us start this topic.
This application combines some of `teqc` and `anubis` features. 
It allows post processed navigation, it integrates a special CGGTTS solutions solver.
All solutions our synthesized as an HTML geodetic report, which is our main solution to this day.
The application is auto-generated for a few architectures, you can directly
[download it from Github.com](https://github.com/georust/rinex/releases)
* [`RNX2BIN`](rnx2bin/) dumps one RINEX or CRINEX into a binary file
made of BINEX Messages.
* [`RNX2CRX`](rnx2crx/) is a RINEX compressor (RINEX to Compact RINEX).
It is a light application that you can combined to `rinex-cli` for a complete workflow.
* [`UBX2RNX`](ublox-rnx/) is an application to generate RINEX files from Ublox receivers.   
This application is currently work in progress.

### Libraries

* [`BINEX`](binex/) BINEX message Encoding and Decoding library
* [`Qc-Traits`](qc-traits/) is a low level library that is shared
between our core libraries to permit consistant behaviors.
* [`RINEX`](rinex/) provides RINEX parsing, formatting and CRINEX support.
It allows post processing of all these file formats
* [`RINEX-Qc`](rinex-qc/) is a our GNSS post processing library.
It allows considering a complex fileset of RINEX, possibly enhanced with
SP3. It generates a geodetic report from all of that.
* [`SINEX`](sinex/) is a core
* [`SP3`](sp3/) High Precision Orbits (by IGS) parsing. 
It allows post processing for PPP.

### Other 

* [`logs`](logs/) is dedicated to store sessions log, if you work within this workspace directly.
* [`tutorials`](tutorials/) is a superset of scripts (Linux/MacOS compatible)
to get started quickly. The examples span pretty much everything our applications allow.
* [`tools`](tools/) are utility scripts and development tools


RINEX-Cli
=========

`rinex-cli` is our main application. Like all applications contained in this repo, it is automatically
generated [upon Release](https://github.com/georust/rinex/releases).

`rinex-cli` supports many options that are closely tied to the [Qc options](./rinex-qc):

- `nav`: will enable post processed navigations
- `cggtts`: enables the special CGGTTS solutions
- `kml`: allows to render the PPP solutions as KML tracks
- `gpx`: allows to render the PPP solutions as GPX tracks

Make sure to read how to [activate the application logs](./tutorials/Logs.md) when
performing advanced operations.

Formats & revisions
===================

The `RINEX` lib supports RINEX V4, that includes the new navigation frames.  
It also supports IONEX and Clock RINEX in their latest revisions. 

The `SP3` lib supports rev D.

File format and applications
============================

This table summarizes all supported formats and how they are managed in the applications.

`Indexing`: gives how this dataset is indexed in their respective core libraries.   
`Qc Indexing`: gives how this dataset is indexed and managed by the [Qc library](rinex-qc/).

| Type                       | Parser            | Writer              |  CLI                 |      Content         | RINEX Indexing       | Timescale  |
|----------------------------|-------------------|---------------------|----------------------|----------------------|----------------------| -----------|
| Navigation  (NAV)          | :heavy_check_mark:| :construction:      |  :heavy_check_mark: :chart_with_upwards_trend:  | Ephemerides, Ionosphere models | [NavKey]() | SV System time broadcasting this message |
| Observation (OBS)          | :heavy_check_mark:| :heavy_check_mark: | :heavy_check_mark:  :chart_with_upwards_trend: | Phase, Pseudo Range, Doppler, SSI | [ObsKey]() | GNSS (any) |
|  CRINEX  (Compressed OBS)  | :heavy_check_mark:| RNX2CRX1 :heavy_check_mark: RNX2CRX3 :construction:  | :heavy_check_mark:  :chart_with_upwards_trend:  |  Phase, Pseudo Range, Doppler, SSI | [ObsKey]() | GNSS (any) |
|  Meteorological data (MET) | :heavy_check_mark:| :heavy_check_mark:  | :heavy_check_mark: :chart_with_upwards_trend:  | Meteo sensors data (Temperature, Moisture..) | [MeteoKey]() | UTC | 
|  Clocks (CLK)              | :heavy_check_mark:| :construction:      | :heavy_check_mark: :chart_with_upwards_trend:  | Precise SV and Reference Clock states |  Epoch | GNSS (any) |
| SP3                        | :heavy_check_mark: | :construction: Work in progress | :heavy_check_mark: :chart_with_upwards_trend: | High precision SV orbital state | Epoch            | GNSS (any) |
|  Antenna (ATX)             | :heavy_check_mark:| :construction:      | :construction:   | Precise RX/SV Antenna calibration | `antex::Antenna` | :heavy_minus_sign: |
|  Ionosphere Maps  (IONEX)  | :heavy_check_mark:|  :construction:     | :heavy_check_mark:  :chart_with_upwards_trend: | Ionosphere Electron density | Epoch | UTC |
|  DORIS RINEX               | :heavy_check_mark:|  :construction:     | :heavy_check_mark:   | Temperature, Moisture, Pseudo Range and Phase observations | Epoch | TAI |
| BINEX                      | :construction: (a)| :construction:      |
|  SINEX  (SNX)              | :construction:    |  :construction:     | :heavy_minus_sign:   | SINEX are special RINEX, they are managed by a dedicated [core library](sinex/) | Epoch | :question: |
|  Troposphere  (TRO)        | :construction:    |  :construction:     | :question:           | Troposphere modeling | Epoch | :question: |
|  Bias  (BIA)               | :heavy_check_mark: |  :construction:    | :question:           | Bias estimates, like DCB.. | Epoch | :question: |

:heavy_check_mark: all revisions supported.   
:construction: : work in progress.  
__CLI__ : supported by the [Qc Library](./rinex-qc)
__CLI__ + :chart_with_upwards_trend: [Qc Reporting](./rinex-qc) may generate data visualization

BINEX (a): some frames are supported, not all of them. Refer to [BINEX](./binex).

Other formats
=============

| Type | Parser             | Writer                          | CLI                                           | Content                         | Record Iteration | Timescale  |
| ---- | ------------------ | ------------------------------- | --------------------------------------------- | ------------------------------- | ---------------- | ---------- |

File name Restrictions
======================

| Format                 | Restriction              |
| ---------------------- | ------------------------ |
| RINEX                  | :heavy_minus_sign:       |
| CRINEX                 | :heavy_minus_sign:       |
| gzip compressed RINEX  | Name must end with `.gz` |
| gzip compressed CRINEX | Name must end with `.gz` |
| .Z compressed RINEX    | Not supported            |
| DORIS RINEX            | :heavy_minus_sign:       |
| gzip compressed DORIS  | Name must end with `.gz` |
| .Z compressed DORIS    | Not supported            |
| SP3                    | :heavy_minus_sign:       |
| gzip compressed SP3    | Name must end with `.gz` |
| .Z compressed SP3      | Not supported            |
| BINEX                  | :heavy_minus_sign:       |
| UBX                    | :heavy_minus_sign:       |

:heavy_minus_sign: No restrictions: file names do not have to follow naming conventions.  

Non readable formats :construction:
===================================

`RINEX-Cli` will soon accept more than readable data.

| Format |     Status     |                     Application                      |
| :----: | :------------: | :--------------------------------------------------: |
|  UBX   | :construction: | Convert your UBX data to RINEX to later post process |
|        | :construction: | Convert your GNSS context to UBX (efficient storage) |
| BINEX  | :construction: |    Convert BINEX streams to readable RINEX files     |
|        | :construction: |  Encode RINEX datasets to BINEX (efficient storage)  |
|  RTCM  | :construction: |     Serve your RINEX/SP3 datasets over RTCM I/O      |

Special Thanks
==============

These tools would not exist without the great libraries written by C. Rabotin, 
[check out his work](https://github.com/nyx-space).  

Some features would not exist without the invaluable help of J. Lesouple, through
our countless discussions. Check out his 
[PhD manuscript (french)](http://perso.recherche.enac.fr/~julien.lesouple/fr/publication/thesis/THESIS.pdf?fbclid=IwAR3WlHm0eP7ygRzywbL07Ig-JawvsdCEdvz1umJJaRRXVO265J9cp931YyI)

Contributions
=============

Contributions are welcomed, do not hesitate to open new issues
and submit Pull Requests through Github.

If you want to take part in active developments, check out our [contribution guidelines and hints](CONTRIBUTING.md) to navigate this library quicker.
