CONTRIBUTING
============

Hello and welcome on board :wave:

Use the github portal to submit PR, all contributions are welcomed.  
Don't forget to run a quick `cargo fmt` prior any submissions, so the CI/CD does not fail
on coding style "issues".

This crate and ecosystem is part of the Georust community. 
You can contact us on [Discord](https://discord.gg/Fp2aape).

Crate architecture
==================

For each supported RINEX types, we have one folder named after that format.  
`src/navigation` is one of those. 

The module contains the record type definition. RINEX file contents vary a lot
depending on which type of RINEX we're talking about.  
For complex RINEX formats like Navigation Data, that module will contain all possible inner types.

Other important structures :
- `src/epoch/mod.rs`: the Epoch module basically provides
hifitime::Epoch parsing methods, because RINEX describes date in non standard formats.
Also, the `Flag` structure is used to mark Observations (valid or invalid).
- `src/constellation.rs` defines GNSS constellations
- `src/constellation/augmentation.rs` : preliminary SBAS support
- `src/sv.rs` defines a Satellite vehicle, which is associated to a constellation
- `src/observable.rs`: defines possible observations like raw phase
- `src/carrier.rs`: defines carrier signals in terms of frequency and bandwidth.
It also contains utilities to identify which GNSS signals we're dealing with,
from an `Observable`.
- `src/hatanaka/mod.rs`: the Hatanaka module contains the RINEX Compressor and Decompressor 
- `src/antex/antenna.rs`: defines the index structure of ANTEX format

Navigation Data
===============

Orbit broadcasted parameters are presented in different form depending on the RINEX revisions
and also may differ in their nature depending on which constellation we're talking about.

To solve that problem, we use a dictionary, in the form of `src/db/NAV/orbits.json`,
which describes all fields per RINEX revision and GNSS constellation.

This file is processed at build time, by the build script and ends up as a static 
pool we rely on when parsing a file. 

The dictionary is powerful enough to describe all revision, the default Navigation Message
is `LNAV`: Legacy NAV message, which can be omitted. Therefore, it is only declared 
for modern Navigation messages.

Introducing a new RINEX type
============================

`src/meteo/mod.rs` is the easiest format and can serve as a guideline to follow.

When introducing a new Navigation Data, the dictionary will most likely have to be updated (see previous paragraph).

GNSS Constellations
===================

Supported constellations are defined in the Constellation Module.  
This structure defines both Orbiting and Stationary vehicles.

On crate feature "sbas", we can determine identify GEO vehicles
in detail, thanks to the rinex/db/SBAS/sbas.json database.  
We don't support undeployed Geostationary vehicles (in advance).

Build scripts
=============

1. Navigation RINEX specs are represented in rinex/db/NAV
2. Geostationary vehicles identification in rinex/db/sbas/sbas.json,
is picked up on "sbas" crate feature.
This follows the L1-CA-PRN Code assignment specifications (see online specs).
3. rinex/db/SBAS/*.wkt contains geographic definitions for most
standard SBAS systems. We parse them as Geo::LineStrings to
define a contour area for a given SBAS system. This gives one method
to select a SBAS from given location on Earth

Crate dependencies
==================

- `qc-traits` and `sinex` are core libraries.
- `rinex` is the central dependency to most other libraries or applications.
- tiny applications like `rnx2crx`, `crx2rnx` and `ublox-rnx` only depend on the rinex crate
- `sp3` is a library that only depends on `rinex` 
- `gnss-rtk` is a library that depends on `rinex`, `sp3` and `rinex-qc`
- `cli` is an application that exposes `rinex-qc`, `gnss-rtk`, `sp3` and `rinex`

External key dependencies:

- `Hifitime` (timing lib) is used by all libraries
- `Nyx-space` (navigation lib) is used by `gnss-rtk`
- `Ublox-rs` (UBX protocol) is used by `ublox-rnx`

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/dependencies.png">
