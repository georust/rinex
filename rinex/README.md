# RINEX

[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![rustc](https://img.shields.io/badge/rustc-1.64%2B-blue.svg)](https://img.shields.io/badge/rustc-1.64%2B-blue.svg)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)

*RINEX* is a crate in the *GeoRust* ecosystem that aims at supporting
most common RINEX formats, for both data analysis and data production,
without performance compromises.

Several applications were and are being built based on RINEX, among those we can cite
[rinex-cli](https://github.com/georust/rinex-cli) which allows parsing, plotting,
processing similary to `teqc`, generating geodetic surveys, solve ppp and cggtts solutions

## Crate features

1. We have one crate feature per RINEX format. 
All RINEX formats are supported and built-in. But you can unlock
specific (usually post-processing) methods that are RINEX format dependent, by activating this crate feature. For example, the `obs` crate feature unlocks signal iteration and combination.

2. `flate2` unlocks seamless Gzip compression and decompression.
Our library is compression and decompression friendly, you'll see this if you read the Hatanaka/CRINEX chapter. `flate2` allows parsing Gzip compressed files directly.

3. The `qc` feature allows generating post-processed reports.
The reports are rendered in HTML and it comprises `teqc` like
file operations as well

4. The `processing` trait is `qc` dependent and goes deeper
in the post processing operations. For example, it unlocks
a Filter designer, to completely rework, repair and post process
datasets

5. `serde`: will unlock Serialization and Deserialization of most of our structures.

5. `full` will activate all known features

## RINEX Revisions

This library supports RINEX V2, V3 and V4 (latest), this includes all new V4 Navigation frames.

## RINEX formats

All RINEX format are supported, but restrictions may apply, especially when generating data. [Refer to the main front-page table right here](https://github.com/georust/rinex) to understand what restriction applies to your usecases.

## Getting started: Parsing

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

## File naming conventions

RINEX File naming conventions are strong and well defined.  
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

// We can always determine a complete V2 filename from correct datasets

// It is impossible for a V3 filename though
```

## File operations

The RINEX library allows several file operations, some were historically introduced by `teqc`

## File generation

The RINEX library is also oriented towards file genration.

## Licensing

Licensed under either of:

* Apache Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
* MIT ([LICENSE-MIT](http://opensource.org/licenses/MIT)
