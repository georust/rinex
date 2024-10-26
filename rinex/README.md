# RINEX

[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![rustc](https://img.shields.io/badge/rustc-1.64%2B-blue.svg)](https://img.shields.io/badge/rustc-1.64%2B-blue.svg)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)

*RINEX* is a *GeoRust* crate that aims at supporting all RINEX formats, enabling advanced GNSS post processing. That means:

- Parsing: text files decoding and datasets interpretation
- Analysis: data post processing, like post processed navigation
- Production: text file formatting (encoding), which is currently
limited because we're still actively working on the Parsing/Analysis steps.

Several RINEX formats exist, among those we support the follwing list:

* Observation RINEX (all revs.)
* CRINEX (all revs.)
* Meteo RINEX (all revs.)
* Navigation RINEX (all Revs.)
* IONEX (all revs.)
* Clock RINEX (All revs.)
* ANTEX

This library was built to support RINEX V4 completely.  
[Refer the front-page table for more detail](https://github.com/georust/rinex).

The main objective of this library is to be a complete toolbox that offers a credible
option to process *RINEX* data. The library offers high level operations and algorithms,
many of them being historically supported or introduced by `TEQc` or `RTKlib`.

## Crate features

The current philosophy is to support all RINEX formats natively.  
You don't have to customize the library to parse any of the supported formats.  
The CRINEX compression algorithm is also supported natively.

We have one crate feature per RINEX format, to either unlock specific methods
or specific Iterators. For example, the `obs` feature is related to Observation RINEX
format, it unlocks signal combination (which is post processing oriented) 
and detailed Iterator to iterate GNSS signals. Another example would be `meteo` which
unlocks `[Rinex::rain_detected]` which is a direct consequence of a specific Iterator.

## RINEX and GZIP

The great `flate2` library allows us to support Gzip compression and decompression.  
Compile our library with this option for seamless support (both ways).  

Note that, parsing a Gzip compressed files requires that filename to be terminated by `.gz`.

## QC feature

The `qc` feature allows:

* high level file operation like [Merge] that were historically introduced by `TEQc`
* Post processing report rendition (currenly in HTML)

It is the root base of our post processing capabilities

## Navigation feature

The `nav` feature is tied to the Navigation RINEX format.  
It not only unlocks specific Iteration method, and also integrate
Ephemeris interpretation and related calculations. That includes the
Kepler equation solver, to be able to actually process the Radio messages.

## Post processing feature(s)

Parsing is typically only the first of many steps in a post processing pipeline. 
RINEX datasets are complex and can rarely be processed "as is".

We have a `processing` feature that is `qc` dependent and goes deeper
in the post processing operations. For example, it unlocks
a Filter designer, to completely rework, repair or reshap prior post processing.

A post processing pipeline will most likely require this feature to be activated.

The RINEX lib stops at the RINEX level, anything that requires any external topic
is out of scope. For that purpose, we developped the `RINEX-QC` library 
in this very repository, which for example, combines the RINEX and SP3 libraries
to permit post processed high precision navigation. If you're interested in
post processing and advanced operations, you are probably more interested by using
this library directly and should move to its dedicated documentation.

## SBAS feature

We don't have much SBAS dedicated features as of yet.  
This option, currenly unlocks a SBAS selection method, that helps you select the appropriate
SBAS system, based on given coordinates on the ground.

## Full feature

The `full` feature will unlock all features at once. 

## Other features

- `serde` unlocks Json serdes
- `log` unlocks debug traces in the Parsing process and Navigation calculations.
It is used by `rinex-cli+debug` for developper verifications.

## Applications

Several applications were and are being built based on RINEX, among those we can cite
[rinex-cli](https://github.com/georust/rinex-cli) which allows parsing, plotting,
processing similary to `teqc`, generating geodetic surveys, solve ppp and cggtts solutions

## Licensing

The RINEX folder is licensed under either of:

* Apache Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
* MIT ([LICENSE-MIT](http://opensource.org/licenses/MIT)

File name conventions and behavior
==================================

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

Getting Started
===============

Reading RINEX files is quick and easy. When the provided file follows standard naming conventions,
the structure definition will be complete: the production context is described by the file name itself.

```rust
use rinex::prelude::RINEX;
let rinex = RINEX::from_path("../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx")
    .unwrap();
```

Our parser is smart and will adapt to the file format you are providing. Read the [`from_file`] API to 
fully understand the little restrictions we have. For example, CRINEX (Compressed RINEX) decompression
is builtin:

```rust
use rinex::prelude::RINEX;
let rinex = RINEX::from_path("../test_resources/CRNX/V1/AJAC3550.21D")
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

In its current form, *RINEX* has few limitations with respect to the filenames you can provide to the Parser.
Refer to the [Rinex::from_file] API documentation, for more detail.

Working with files that do not follow standard naming conventions, or Stream interface,
we have no means to determine the [FileProductionAttributes]:

```rust
```

When working with files that follow the V2 standard naming conventions, some of the file production setup
cannot be determined and remains unknown

```rust

# but you have means to change that
```

We developped a smart [FileProductionAttributes] guesser, that will guess those from the actual file content.
This may apply to two scenarios:

* guessing accurate *FileProductionAttributes* when working with files that do not follow
standard naming conventions but contain accurate data, and actually use this library to properly rename those
* stay focused on data production (actual data symbols) in production context, and use the guesser to
auto determine an accurate file name.

Exemple:

```rust
// V2/short filenames are incomplete

<<<<<<< HEAD
// We can always determine a complete V2 filename from correct datasets

// It is impossible for a V3 filename though
```

