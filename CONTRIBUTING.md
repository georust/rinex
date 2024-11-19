CONTRIBUTING
============

Hello and welcome :wave:

The GeoRust RINEX toolbox is a powerful & somewhat complete toolbox,
at least, it aims to be! it is also fully open and all contributions are welcome. 
Use the github portal for that purpose. Don't forget to run a quick `cargo fmt` linter
prior submissions, so the CI/CD jobs does not fail for irrelevant reasons.  

For questions you can either open a discussion on Github.com or drop us a message
on [Discord](https://discord.gg/Fp2aape) (prefer the RINEX channel).

Toolbox
=======

This repository is a complete toolbox because it does not only host libraries.
`rinex-cli` is the main application as of today and it does not come with a GUI.  
We still have quite a lot to achieve in terms of processing, and we consider we will focus
on GUI development afterwards. If you're interested in helping us taking this step quicker: do not hesitate.

Post-processed navigation is complex and therefore, does not rely on the RINEX lib on its own. 
In fact, RINEX is just the (only) file format that allows saving the data context that we can later on
study and process. Navigation being just one of many applications we can use GNSS for. Our applications
are not only dedicated to navigation, they allow more than that.

The main dependencies for RINEX processing is summarized with:

* GNSS-RTK for the core navigation calculations
* ANISE for the frame and space model
* Nyx-space for advanced navigation features
* Hifitime for all timing aspects, which is a key element

Scroll down for further detail for each repo:

* [rinex](#rinex) the RINEX library
* [sp3](#sp3) parser to permit PPP
* [rinex-qc](#rinex-qc) for RINEX and post processed navigation

RINEX
======

The RINEX library is mainly a parser library. We have still a few
steps to take to fully support data formatting. In terms of data capabilities,
it is fair to say it has a lot of interesting features, yet it is not perfect
and key processing options are still lacking.

## Crate Architecture

In terms of architecture, the library crate is well organized. The
current work aims at further improving this. It should facilitate new comers. 
Because RINEX has several formats, the library is lengthy. Yet we are able
to achieve a simple and unique interface and support all RINEX formats.
Also, the new NAV V4 frames are supported, which is still kind of rare today.

For each RINEX format, we have one submodule, for example `observation` for
Observation RINEX.

The Hatanaka module contains the CRINEX decompressor and RINEX compressor objects.

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

Adding new SBAS vehicles
========================

To add a newly launched SBAS vehicles, simply add it to the
gnss-rs/data/sbas.json database.

This database is auto integrated to this library to provide
detailed SBAS supports. The only mandatory fields (in the databse) are:
- the "constellation" field
- the SBAS "prn" field (which is 100 + prn number)
- "id": the name of that vehicle, for example "ASTRA-5B"
- "launched\_year": the year this vehicle was launched

Other optional fields are:
- "launched\_month": month ths vehicle was launched
- "launched\_day": day of month this vehicle was launched

We don't support undeployed vehicles (in advance).

Modify or add Navigation Frames
===============================

Navigation frames are desc

The build script is rinex/build.rs.

It is responsible for building several important but hidden structures.

1. Navigation RINEX specs, described by rinex/db/NAV
2. Geostationary vehicles identification in rinex/db/sbas/sbas.json,
that follows the L1-CA-PRN Code assignment specifications (see online specs).
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

SP3
===

SP3 parser library was developped to support that particular file format, which is required
in PPP post processed scenario. `rinex-qc` accepts such an input, and can stack them to the context, `rinex-cli` can post process
the data context.

SP3 parser is less dense that RINEX because it is a unique format and answers the Navigation part of the RINEX dataset.
