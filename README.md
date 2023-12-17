RINEX 
=====

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)

[![minimum rustc: 1.64](https://img.shields.io/badge/minimum%20rustc-1.64-blue?logo=rust)](https://www.whatrustisit.com)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 

Rust tool suites to parse, analyze and process [RINEX Data](https://en.wikipedia.org/wiki/RINEX).

Our Wiki contains [several tutorials and applications](https://github.com/georust/rinex/wiki): it will get you started quickly.

For any question or problems you may experience:

- open a new issue
- drop us a message [on Discord](https://discord.gg/Fp2aape)

## Advantages :rocket: 

- Fast
- Open sources
- Native Hatanaka decompression and compression
- Seamless .gzip decompression with `flate2` compilation feature
- RINEX V4 full support, that includes modern Navigation messages
- Meteo RINEX full support
- IONEX (2D) support, partial 3D support
- Clock RINEX partial support: to be concluded soon
- File merging, splitting and pre processing
- Modern constellations like BeiDou, Galileo and IRNSS
- Supported time scales are GPST, BDT, GST, UTC
- Supports many SBAS, refer to online documentation
- Full support of Military codes : if you're working with such signals you can
at least run a -qc analysis, and possibly the position solver once it is merged 
- Supports high precision RINEX (scaled phase data with micro cycle precision)
- RINEX post processing like SNR, DCB analysis, Broadcast ephemeris interpolation,
high precision orbit interpolation (SP3)..
- RINEX-qc: statistical analysis that you can request in the "cli" application directly.
Analysis can run on modern GNSS signals and SP3 high precision data.
Emulates "teqc" historical application.
- An SPP/PPP position solver (under development), in the form of the "gnss-rtk" library that you can
summon from the "cli" application directly.

## Disadvantages :warning:

- QZNSST is represented as GPST at the moment.
- We're waiting for Hifitime V4 to support GLONASST and IRNSST.   
Until then, orbital calculations on these systems are not feasible.   
In other term, positioning is not feasible and you're limited to basic analysis. 
- These tools are oriented towards the latest revisions of the RINEX format.
RINEX4 is out and we already support it. 
Some minor features in the RINEX2 or 3 revisions may not be supported.
- We currently do not support the Channel Number Pseudo Observable, to very precisely
describe how the signals were sampled (RINEX > 2).
- Our command line applications do not accept BINEX or other proprietary formats
- File production is not fully concluded to this day. We're currently focused
on RINEX post processing rather than RINEX data production. Do not hesitate to fork and submit
your improvements

## Architecture 

* [`rinex`](rinex/) is the core library 
* [`rinex-cli`](rinex-cli/) : an application dedicated to RINEX post processing.
It supports some of `teqc` operations.   
The application is auto-generated for a few architectures, download it from the 
[release portal](https://github.com/gwbres/rinex/releases)

* [`sp3`](sp3/) High Precision Orbits (by IGS) 
* [`rnx2crx`](rnx2crx/) is a RINEX compressor (RINEX to Compact RINEX)
* [`crx2rnx`](crx2rnx/) is a CRINEX decompresor (Compact RINEX to RINEX)
* [`rnx2cggtts`](rnx2cggtts/) post processes RINEX data and resolves PVT that we
wrap in CGGTTS format which is dedicated to (remote) clock comparison 
* [`rinex-qc`](rinex-qc/) is a library dedicated to RINEX files analysis 
* [`qc-traits`](qc-traits/) declares Traits that are shared between `rinex` and `rinex-qc`
* [`sinex`](sinex/) SNX dedicated core library

* [`ublox-rnx`](ublox-rnx/) is an application intended to generate RINEX Data
from raw uBlox GNSS receiver frames. This application is work in progress at the moment.

## Other tools and relevant Ecosystems

* [Nyx-space](https://github.com/nyx-space/nyx)
* [Hifitime](https://github.com/nyx-space/hifitime)
* [CGGTTS](https://github.com/gwbres/cggtts)
* [GNSS definitions in Rust](https://github.com/rtk-rs/gnss)

RINEX formats & applications
============================

| Type                       | Parser            | Writer              |  CLI                 | UBX                  |          Content         | Record browsing      |
|----------------------------|-------------------|---------------------|----------------------|----------------------|--------------------------| ---------------------|
| Navigation  (NAV)          | :heavy_check_mark:| Ephemeris :construction: V4 :construction: |  :heavy_check_mark: :chart_with_upwards_trend:  | :construction:       | Orbit parameters, Ionospheric models.. | Epoch iteration |
| Observation (OBS)          | :heavy_check_mark:| :heavy_check_mark: | :heavy_check_mark:  :chart_with_upwards_trend: |  :construction:  | Phase, Pseudo Range, Doppler, SSI | Epoch iteration |
|  CRINEX  (Compressed OBS)  | :heavy_check_mark:| RNX2CRX1 :heavy_check_mark: RNX2CRX3 :construction:  | :heavy_check_mark:  :chart_with_upwards_trend:  |  :construction:  | Phase, Pseudo Range, Doppler, SSI | Epoch iteration |
|  Meteorological data (MET) | :heavy_check_mark:| :heavy_check_mark:  | :heavy_check_mark: :chart_with_upwards_trend:  | :construction:  | Meteo sensors data (Temperature, Moisture..) | Epoch iteration |  
|  Clocks (CLK)              | :heavy_check_mark:| :construction:      | :construction:   |:construction: | Clock comparison |  Epoch iteration |
|  Antenna (ATX)             | :heavy_check_mark:| :construction:      | :construction:   |:construction: | Antenna calibration data | Sorted by `antex::Antenna` |
|  Ionosphere Maps  (IONEX)  | :heavy_check_mark:|  :construction:     | :heavy_check_mark:  :chart_with_upwards_trend: |:construction: | Ionosphere Electron density | Epoch iteration |
|  SINEX  (SNX)              | :construction:    |  :construction:     | :heavy_minus_sign:   |:construction: | SINEX are special RINEX, they are managed by a dedicated [core library](sinex/) | Epoch iteration |
|  Troposphere  (TRO)        | :construction:    |  :construction:     | :question:           |:construction: | Troposphere modeling | Epoch iteration | 
|  Bias  (BIA)               | :heavy_check_mark: |  :construction:    | :question:           |:construction: | Bias estimates, like DCB.. | Epoch iteration | 

:heavy_check_mark: means all revisions supported   
:construction: : Work in Progress   
__CLI__ + :chart_with_upwards_trend: means the [cli app](rinex-cli/README.md) provides one or several visualizations

The [cli app](rinex-cli/README.md) accepts more than RINEX input, for example SP3 (high precision orbits) are accepted.

File formats
============

| Format                 | File name restrictions            |    Support                         |
|------------------------|-----------------------------------|------------------------------------|
| RINEX                  | :heavy_minus_sign:                | :heavy_check_mark:                 |
| CRINEX                 | :heavy_minus_sign:                | :heavy_check_mark:                 | 
| gzip compressed RINEX  | Name must end with `.gz`          | `--flate2` feature must be enabled |
| gzip compressed CRINEX | Name must end with `.gz`          | `--flate2` feature must be enabled |
| SP3                    | :heavy_minus_sign:                | :heavy_check_mark:                 | 
| gzip compressed SP3    | Name must end with `.gz`          | `--flate2` feature must be enabled | 

:heavy_minus_sign: No restrictions: file names do not have to follow naming conventions.  

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
