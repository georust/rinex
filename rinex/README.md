# RINEX

[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![rustc](https://img.shields.io/badge/rustc-1.64%2B-blue.svg)](https://img.shields.io/badge/rustc-1.64%2B-blue.svg)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)
[![Rust](https://github.com/georust/rinex/actions/workflows/daily.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/daily.yml)

*RINEX* is a *GeoRust* crate that aims at supporting all RINEX formats, enabling advanced GNSS post processing. That means:

- Parsing: text files decoding and datasets interpretation
- Analysis: data post processing, like post processed navigation
- Production: text file formatting (encoding), which is currently
limited because we're still actively working on the Parsing/Analysis steps.

Several RINEX formats exist, among those we support:

* Observation RINEX
* CRINEX (Compact RINEX)
* Navigation RINEX
* Meteo RINEX
* Clock RINEX
* IONEX
* ANTEX

[Refer to the front-page table](https://github.com/georust/rinex?tab=readme-ov-file#formats--revisions)
for more details on supported formats.

The main objective of this library is to be a complete and credible toolbox.
Nowadays it supports high level operations and algorithms, many of them being historically
introduced either by `TEQc` or `RTKlib`.

## Crate features

The RINEX library supports all RINEX formats and revisions natively,
that includes the CRINEX compression algorithm.

We have one crate feature per file format, to either unlock specific methods
or Iterators. It allows going deeper into one topic. For example, the `obs` feature is 
related to Observation RINEX files and unlocks the signal combination
method, which is a post processing method. Another example, would be the `meteo` feature
which unlocks `[Rinex::rain_detected]` which is a direct exploitation of one of its specific Iterators.

## RINEX and Gzip compression

The great `flate2` library allows us to support Gzip compression and decompression.  
Compile our library with this option for seamless support (both ways).  

## QC feature

The `qc` feature allows high level operation like [Merge],
that were historically introduced by `TEQc`.

It is the root base of our post processing capabilities.

## Navigation feature

The `nav` feature is tied to the Navigation RINEX format.  

It not only unlocks specific Iteration methods, but also integrates Ephemeris
interpretation and associated calculations, mostly the Kepler solvers
that allows navigation from radio messages. It is the root base to radio based
post processed navigation using RINEX.

If you're interested in post processed navigation using the RINEX library, you will
need to activate this feature.

## Post processing feature(s)

Parsing is typically only the first of many steps in a post processing pipeline. 
RINEX datasets are complex and can rarely be processed "as is".

We have a `processing` feature that is `qc` dependent and goes deeper
in the post processing operations. For example, it unlocks
[Preprocessing], which allows reshaping and reworking datasets easily, prior actual processing. [Repair] is another useful traits that is very often needed when surveying in the real world.

A post processing pipeline will most likely require this feature to be activated.

Although *RINEX* knows how to physically interprate a dataset, anything
that is beyond that is out of scope of this library. 

You should refer to the [GNSS Qc library](https://docs.rs/gnss-qc/latest/gnss_qc/) for
that very purpose. It allows advanced exploitations of RINEX.
Only this library may answer the requirements of GNSS post processing.

If you're interested
in post processed Navigation for example, you are probably more interested in
using the `Qc` library than simply *RINEX*. 

## SBAS and Geostationary :artificial_satellite:

There is no feature related to geostationary satellites. The RINEX file format treats them
like any other satellite vehicle. Therefore, the `nav` feature will support them like any other.

## Full feature

The `full` feature will unlock all features at once. 

## Other features

- `serde` unlocks Json serdes
- `log` unlocks debug traces in the Parsing process and Navigation calculations.
It is used by `rinex-cli+debug` for developper verifications.

## Applications

Several applications were and are being built based on RINEX, among those we can cite
[the RTK-rs framework](https://github.com/rtk-rs) with:
- [rinex-cli](https://github.com/rtk-rs/rinex-cli) that allows RINEX post processing.
It has options similar to TEQc and can solve PVT solutions like RTKlib.
- [crx2rnx](https://github.com/rtk-rs/crx2rnx) which aims at becoming a modern replacement of the
historical tool
- [rnx2crx](https://github.com/rtk-rs/rnx2crx) which aims at becoming a modern replacement of the
historical tool

## Licensing

The RINEX folder is licensed under either of:

* Apache Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
* MIT ([LICENSE-MIT](http://opensource.org/licenses/MIT)

Getting Started
===============

Reading RINEX files is quick and easy. When the provided file follows standard naming conventions,
the structure definition will be complete: the production context is described by the file name itself.

```rust
use rinex::prelude::Rinex;
let rinex = Rinex::from_file("../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx")
    .unwrap();
```

Our parser is smart and will adapt to the file format you are providing. Read the [`from_file`] API to 
fully understand the little restrictions we have. For example, CRINEX (Compressed RINEX) decompression
is builtin:

```rust
use rinex::prelude::Rinex;
let rinex = Rinex::from_file("../test_resources/CRNX/V1/AJAC3550.21D")
    .unwrap();
```

When compiled with `flate2`, gzip files can also be parsed directly.  
The ultimate gzip + CRINEX double compression is then becomes natively supported.

*RINEX* allows working with abstract Readable interface as well, that means parsing
local files is just one particular case of the data we can parse. In any case
your interface will always have to either

* stream plain UTF8 RINEX 
* or CRINEX encoded UTF8
* or valid Gzip bianry data

and it cannot change once the parser has been built: you need to create a new parser to adapt
to a new scenario.

When working with files that do not follow standard naming conventions, or directly
from Stream interface, we have no means to determine the [FileProductionAttributes].
This will most likely impact data production scenarios.

When working with files that follow the V2 standard naming conventions, some of the file production setup
cannot be determined and remains unknown

We developped a smart [FileProductionAttributes] guesser, that will guess those from the actual file content.
This may apply to two scenarios:

* guessing accurate *FileProductionAttributes* when working with files that do not follow
standard naming conventions but contain accurate data, and actually use this library to properly rename those
* stay focused on data production (actual data symbols) in production context, and use the guesser to
auto determine an accurate file name.

File production
===============

[ProductionAttributes] stores the information representing the file production context.  
This information is described by file names that follow standard conventions.
If you come from a file that does not follow these conventions, we have no means to fully
determine a V3 (lengthy) file name. We developped a smart guesser to figure out
most of the fields, which particularly applies to high quality datasets. In other words,
you can use the RINEX library as a file name convention generator. If your data is realistic,
the smart guess will figure the proper filename to use.

Coming from randomly named filenames, V3 (lengthy) filenames can never be fully be figured out,
you need to provide some [ProductionAttributes] fields yourself.

