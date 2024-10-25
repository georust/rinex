# RINEX

[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![rustc](https://img.shields.io/badge/rustc-1.64%2B-blue.svg)](https://img.shields.io/badge/rustc-1.64%2B-blue.svg)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)

*RINEX* is a *GeoRust* crate that aims at supporting all RINEX formats,
enabling advanced GNSS post processing. That means:

- Parsing: text file and datasets decoding
- Analysis: *RINEX* data post processing, navigation, etc..
- Production: text file formatting

Several RINEX formats exist, nowadays we support all of them, at least on the parsing side.   
Exotic formats may still lack some features. File production is not fully operational either:
we are focused on post processing.

The main objective of this library is to be a complete toolbox that offers a credible
option to process *RINEX* data. The library offers high level operations and algorithms,
many of them being historically supported by `TEQc` or `RTKlib`.

## Parser versus file naming conventions

The parser does not care about the file names. You can safely parse
files that do not follow standard conventions.

## Production versus file naming conventions

When producing data, you have several options. The information is contained
in the [ProductionAttributes] structure. 
If you come from a file that did not follow naming conventions, there is no means
for this structure to be complete. A smart guesser was developped, to figure out
most of these fields, even coming from poorly named files. This is particularly true
if your data is consistent and has good quality. In other words, this library
can serve as a standard filename generator.

Note that V3 (=long) filenames cannot be entirely figured out, if you come from a file
that did not follow standard naming conventions. Only V2 (=short) filenames can.

In other scenarios, customizing [ProductionAttributes] lets you define your own setup
and how you want to describe your context.

## RINEX Standards

This library was built to support RINEX V4 completely, but efforts
were made to also support older revisions too. The new Navigation frames are fully supported.

Refer to the [front page table](https://github.com/georust/rinex/#rinex-standards)
for more detail.

## RINEX and GZIP

The great `flate2` library allows us to support Gzip compression and decompression.  
Compile our library with this option for seamless support (both ways).  

Note that, parsing a Gzip compressed files requires that filename to be terminated by `.gz`.

## CRINEX compression algorithm

The CRINEX (Compact RINEX) compression algorithm is supported both ways.
We don't have options to unlock this feature: it is builtin.

## Crate features

*RINEX* has one compilation option per RINEX format. For example, "obs" for OBSERVATION RINEX.   
When you unlock this option, you obtain dedicated Iteration and specific methods. 
For example:

- only the `obs` feature gives you an option to Iterate on a GNSS signal basis.
- the `meteo` feature unlocks a `rain_detected()` method, that is directly based
on specific Iterators.

## QC feature

The `qc` toolbox (other libraries within this repository) allows the basis of GNSS processing
and specific RINEX features. If you compile *RINEX* with this option, you get a couple of
methods that enable File operation and processing (basic). The historical `TEQc` methods
are wrapped in this feature: for example, File Merging.

## GNSS processing

Parsing is not that interesting after all. What we want is to actually do something from this data.    
The main target of the RINEX file format is to describe what is necessary to precise navigation on the ground.  
To permit that, we have the `processing` feature that enables:

- A filter designer: to preprocess your datasets efficiently (for example: filter by constellation).
- Data resampling
- The basis of satellite based navigation.

The processing feature is closely tied to the qc feature.

## SBAS feature

We don't have much SBAS dedicated features as of yet.  
This option, currenly unlocks a SBAS selection method, that helps you select the appropriate
SBAS system, based on given coordinates on the ground.

## Full feature

RINEX post processing is usually performed on computers where performance is rarely an issue.  
If your application is post processed GNSS navigation, you're probably better of turning on the
`full` feature, to unlock all previous options at once.

## Licensing

The RINEX folder is licensed under either of:

* Apache Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
* MIT ([LICENSE-MIT](http://opensource.org/licenses/MIT)
