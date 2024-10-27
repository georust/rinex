RINEX 
=====

[![Rust](https://github.com/georust/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/rust.yml)
[![Rust](https://github.com/georust/rinex/actions/workflows/daily.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/daily.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)

[![minimum rustc: 1.64](https://img.shields.io/badge/minimum%20rustc-1.64-blue?logo=rust)](https://www.whatrustisit.com)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/georust/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/georust/rinex/blob/main/LICENSE-MIT) 

Rust tool suites to parse, analyze and process [RINEX](https://en.wikipedia.org/wiki/RINEX) and GNSS data.

The [Wiki pages](https://github.com/georust/rinex/wiki) contain all documentation and example applications of this toolbox.  

Use [Github Issues](https://github.com/georust/rinex/issues) to report bugs and other malfunctions.  
You can also open a [Discussion](https://github.com/georust/rinex/discussions) or leave us a message [on Discord](https://discord.gg/Fp2aape).

## Advantages :rocket: :artificial_satellite:

- Fast
- Renders High level Geodetic survey reports
- Fast PPP solver
- Open sources: read and access all the code!
- Self sustained examples and tutorials: data hosted within this repo
- All modern GNSS constellations, codes and signals
  - Surveying with GPS, Galileo, BeiDou and QZSS
- Time scales: GPST, QZSST, BDT, GST, UTC, TAI
- Efficient seamless compression and decompression
- RINEX V4 full support
- All RINEX formats supported (see following table)
- High Precision Clock RINEX products (for PPP)
- High Precision Orbital [SP3 for PPP](https://docs.rs/sp3/1.0.7/sp3/)
- DORIS (special RINEX)
- Many pre-processing algorithms including Filter Designer
- Several file operations: merging, splitting, time binning (batch)
- Post processing:
  - [Position solver](https://github.com/georust/rinex/wiki/Positioning)
  - [CGGTTS solver](https://github.com/georust/rinex/wiki/CGGTTS)

## Disadvantages :warning:

- Navigation is currently not feasible with Glonass and IRNSS (applications/ ppp solver).
- QZSS has not been tested in the PPP solver yet
- PPP solver and Navigation in general using SBAS is not 100 % feasible yet
- RTK navigation is not feasible yet (work in progress) 
- Our applications do not accept proprietary formats like Septentrio for example
- BINEX support is currently work in progress.
Library exists and works, not integrated to applications yet.

## Repository 

* [`rinex`](rinex/) is the core library 
* [`rinex-cli`](rinex-cli/) is a command line application to process RINEX, SP3 and soon Ublox, and dedicated to typical GNSS post processing.  
It is growing as some sort of Anubis/Teqc/Glab combination. No GUI currently available, this will be developed later.   
It integrates a PVT and CGGTTS solutions solver.  
The application is auto-generated for a few architectures, you can directly
[download it from Github.com](https://github.com/georust/rinex/releases)
* [`tutorials`](tutorials/) is a superset of scripts (Linux/MacOS compatible)
to get started quickly. The examples span pretty much everything our applications allow.
* [`sp3`](sp3/) High Precision Orbits (by IGS) 
* [`binex`](binex/) BINEX Encoding and Decoding library
* [`rnx2crx`](rnx2crx/) is a RINEX compressor (RINEX to Compact RINEX)
* [`crx2rnx`](crx2rnx/) is a CRINEX decompresor (Compact RINEX to RINEX)
* [`rinex-qc`](rinex-qc/) is a library dedicated to RINEX files analysis 
* [`qc-traits`](qc-traits/) declares Traits that are shared between `rinex` and `rinex-qc`
* [`sinex`](sinex/) SNX dedicated core library
* [`ublox-rnx`](ublox-rnx/) is an application to generate RINEX files from Ublox receivers.   
This application is currently work in progress
* [`tools`](tools/) are utility scripts and development tools
* [`logs`](logs/) is dedicated to store session logs, if you work within this workspace directly.

## Relevant Ecosystem

* [IGS Network](https://network.igs.org/): browse and monitor any IGS station status
* [Nyx-space](https://github.com/nyx-space/nyx): Navigation and Orbital calculations in Rust
* [Hifitime](https://github.com/nyx-space/hifitime): Precise Time and Timescale support in Rust
* [CGGTTS](https://github.com/gwbres/cggtts): Common View Time Transfer file format, in Rust
* [Geo](https://github.com/georust/geo): Geospatial primitives and algorithms, in Rust
- [RTK-RS](https://github.com/rtk-rs/gnss-rtk): Precise Positioning (calculations) in Rust
* [GNSS definitions](https://github.com/rtk-rs/gnss), in Rust

## Citation and referencing

If you need to reference this work, please use the following model:

`GeoRust RINEX Team (2023), RINEX: analysis and processing (Apache-2/MIT), https://georust.org`

RINEX-Cli
=========

`rinex-cli` is our main application, build it without any features to obtain its smallest form.

Available options are:

- `kml`: allows formatting PPP solutions as KML tracks
- `gpx`: allows formatting PPP solutions as GPX tracks
- `cggtts`: enable CGGTTS solutions solver

`rinex-cli` always generates logs, whether you see them or not is up to your environment.  
But activating the `log` feature of `rinex-cli` actually turns internal dependency logging 
(like the `RINEX` lib itself) for debugging / testing purposes.

Formats & revisions
===================

The `RINEX` lib supports RINEX V4, including the new Navigation frames.  
It also supports IONEX and Clock RINEX in their latest revisions. 

The `SP3` lib supports rev D.

RINEX Format and applications
=============================

This table summarizes the RINEX format we support. 
It also gives a better understanding of what they contain and what they're used for.   
`Record Indexing` gives the internal structure that is used as the Epoch Indexer, in the *RINEX* lib. 
In otherwords, this is how this particular type of dataset is sorted and iterated.  
*Timescale* gives the general Hifitime Timescale the Epochs are expressed in.  
It is important to understand that as well.

| Type                      | Parser             | Writer                                                  | CLI                                            | Content                                                                         | Record Indexing  | Record Iteration                         | Timescale |
| ------------------------- | ------------------ | ------------------------------------------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------- | ---------------- | ---------------------------------------- |
| Navigation  (NAV)         | :heavy_check_mark: | :construction:                                          | :heavy_check_mark: :chart_with_upwards_trend:  | Ephemerides, Ionosphere models                                                  | Epoch            | SV System time broadcasting this message |
| Observation (OBS)         | :heavy_check_mark: | :heavy_check_mark:                                      | :heavy_check_mark:  :chart_with_upwards_trend: | Phase, Pseudo Range, Doppler, SSI                                               | Epoch            | GNSS (any)                               |
| CRINEX  (Compressed OBS)  | :heavy_check_mark: | RNX2CRX1 :heavy_check_mark: RNX2CRX3 :heavy_check_mark: | :heavy_check_mark:  :chart_with_upwards_trend: | Phase, Pseudo Range, Doppler, SSI                                               | Epoch            | GNSS (any)                               |
| Meteorological data (MET) | :heavy_check_mark: | :heavy_check_mark:                                      | :heavy_check_mark: :chart_with_upwards_trend:  | Meteo sensors data (Temperature, Moisture..)                                    | Epoch            | UTC                                      |
| Clocks (CLK)              | :heavy_check_mark: | :construction:                                          | :heavy_check_mark: :chart_with_upwards_trend:  | Precise SV and Reference Clock states                                           | Epoch            | GNSS (any)                               |
| Antenna (ATX)             | :heavy_check_mark: | :construction:                                          | :construction:                                 | Precise RX/SV Antenna calibration                                               | `antex::Antenna` | :heavy_minus_sign:                       |
| Ionosphere Maps  (IONEX)  | :heavy_check_mark: | :construction:                                          | :heavy_check_mark:  :chart_with_upwards_trend: | Ionosphere Electron density                                                     | Epoch            | UTC                                      |
| DORIS RINEX               | :heavy_check_mark: | :construction:                                          | :heavy_check_mark:                             | Temperature, Moisture, Pseudo Range and Phase observations                      | Epoch            | TAI                                      |
| SINEX  (SNX)              | :construction:     | :construction:                                          | :heavy_minus_sign:                             | SINEX are special RINEX, they are managed by a dedicated [core library](sinex/) | Epoch            | :question:                               |
| Troposphere  (TRO)        | :construction:     | :construction:                                          | :question:                                     | Troposphere modeling                                                            | Epoch            | :question:                               |
| Bias  (BIA)               | :heavy_check_mark: | :construction:                                          | :question:                                     | Bias estimates, like DCB..                                                      | Epoch            | :question:                               |

:heavy_check_mark: all revisions are supported   
:construction: : Work in Progress   

__CLI__ : possibility to [load this format](https://github.com/georust/rinex/wiki/file-loading) in the apps.  
__CLI__ + :chart_with_upwards_trend: : possibility to [project or extract and plot](https://github.com/georust/rinex/wiki/graph-mode) this format.


Other formats
=============

`RINEX-Cli` accepts more than RINEX data.  

| Type | Parser             | Writer                          | CLI                                           | Content                         | Record Iteration | Timescale  |
| ---- | ------------------ | ------------------------------- | --------------------------------------------- | ------------------------------- | ---------------- | ---------- |
| SP3  | :heavy_check_mark: | :construction: Work in progress | :heavy_check_mark: :chart_with_upwards_trend: | High precision SV orbital state | Epoch            | GNSS (any) |

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
