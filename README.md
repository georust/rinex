RINEX 
=====

[![Rust](https://github.com/rtk-rs/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/rtk-rs/rinex/actions/workflows/rust.yml)
[![Rust](https://github.com/rtk-rs/rinex/actions/workflows/daily.yml/badge.svg)](https://github.com/rtk-rs/rinex/actions/workflows/daily.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)

[![License](https://img.shields.io/badge/license-MPL_2.0-orange?style=for-the-badge&logo=mozilla)](https://github.com/rtk-rs/sp3/blob/main/LICENSE)

[RINEX (Receiver Independent EXchange)](https://en.wikipedia.org/wiki/RINEX) parser and writer.  
The RINEX format is fully open source and is specified to answer the requirements of navigation and precise navigation.

Use [Github Issues](https://github.com/rtk-rs/rinex/issues) to report issues while using this library.
can also open a [Discussion](https://github.com/rtk-rs/rinex/discussions) or leave us a message [on Discord](https://discord.gg/duETmeGc).

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

- Navigation is currently not feasible with Glonass and IRNSS
- File production might lack some features, mostly because we're currently focused on data processing

## Citation and referencing

If you need to reference this work, please use the following model:

`RTK-rs Team (2025), RINEX: analysis and processing (MPLv2), https://github.com/rtk-rs`

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

Contributions
=============

Contributions are welcomed, do not hesitate to open new issues
and submit Pull Requests through Github.

If you want to take part in active developments, check out our [contribution guidelines and hints](CONTRIBUTING.md) to navigate this library quicker.
