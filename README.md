# RINEX 
Rust package to parse and analyze Rinex files

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT)     
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)
[![Coverage Status](https://img.shields.io/coveralls/github/gwbres/rinex/main?style=flat-square)](https://coveralls.io/github/gwbres/rinex?branch=main)

[![codecov](https://codecov.io/gh/gwbres/rinex/branch/main/graph/badge.svg)](https://codecov.io/gh/gwbres/rinex)

Many RINEX file types exist, 
the `RinexType` enum (refer to API) 
describes the types of RINEX currently supported:

* `RinexType::NavigationMessage` (NAV) messages
* `RinexType::ObservationData` (OBS) data
* `RinexType::MeteoData` (Meteo) data

`RINEX` files contain a lot of data and this library is capable of parsing all of it.   
To fully understand how to operate this lib, refer to the `RinexType` section you are interested in.

Link to the [official API](https://docs.rs/rinex/latest/rinex/index.html)

### Supported RINEX revisions

* 2.00 ⩽ v < 4.0    Tested 
*             v = 4.0    should work, not garanteed at the moment

### RINEX file naming convention

This parser does not check whether the provided local file
follows the RINEX naming convention or not.
You can parse anything and behavior adapts to the actual file content.

### Known weaknesses

* Compressed RINEX (**CRINEX**)   
this lib is not able to decompress CRINEX files at the moment.   
you should manually decompress your RINEX files prior attempting
parsing.    
In that scenario, `rinex.record` is set to _None_, and
the header is fully parsed.

* Weird RINEX content:    
this parser is not able to parse the RINEX body if the provided
RINEX content only has a single epoch.    
In that scenario, `rinex.record` is set to _None_, and
the header is fully parsed.

## Getting started 

The ``Rinex::from_file`` method parses a local `RINEX` file:

```rust
let path = std::path::PathBuf::from("data/NAV/V2/amel0010.21g");
let rinex = rinex::Rinex::from_file(&path).unwrap();
```

The `data/` folder contains short but relevant RINEX files, 
spanning almost all revisions and supported types, mainly for CI purposes.

For data analysis and manipulation, you must refer to the
[official RINEX definition](https://files.igs.org/pub/data/format/)

This [interactive portal](https://gage.upc.edu/gFD/) 
is also a nice interface to
discover or figure things out. 

### Header & general information

The `header` contains high level information.   
`Comments` are currently discarded and not exposed by the parser.   

```rust
println!("{:#?}", rinex.header);
```

This includes `Rinex`:
* revision number
* GNSS constellation
* possible file compression infos
* recorder & station infos
* hardware, RF infos
* and much more

```rust
println!("{:#?}", rinex.header.version);
assert_eq!(rinex.header.constellation, Constellation::Glonass)
println!("{:#?}", rinex.header.crinex)
println!("pgm: \"{}\"", rinex.header.program);
println!("run by: \"{}\"", rinex.header.run_by);
println!("station: \"{}\"", rinex.header.station);
println!("observer: \"{}\"", rinex.header.observer);
println!("{:#?}", rinex.header.leap);
println!("{:#?}", rinex.header.coords);
```

## RINEX record

The `Rinex` structure comprises the `header` previously defined,
and the `record` which contains the data payload.

The _record_ is optionnal at the moment to handle `CRINEX`
Observation Data that we are not able to parse directly.

The `record` is a complex structure of HashMap (_dictionaries_)
whose definition varies with the type of RINEX file.   
Refer to its definition in the API and the specific documentation down below,
for the type of file you are interested in.

`record` is always first indexed by an `epoch`,
that is, data is sorted by the sampling timestamp.

## `Epoch` object

[Epoch structure](https://docs.rs/rinex/latest/rinex/epoch/index.html)

`epoch` is a `chrono::NaiveDateTime` object validated by an
`EpochFlag`.    
A valid epoch is validated with `EpochFlag::Ok`, refer to specific API.

To demonstrate how to operate the `epoch` API, we'll take 
a Navigation Rinex file as an example. 

First, let's grab the record:

```rust
let rinex = rinex::Rinex::from_file("data/amel0010.21g")
  .unwrap();
let record = rinex.record
  .unwrap() // option<record> unwrapping, None in case of CRINEX
    .as_nav() // NAV record unwrapping
    .unwrap();
```

`epochs` serve as keys of the first hashmap.  
The `keys()` iterator is the easiest way to to determine
which _epochs_ were idenfitied.   
Here we are only interested in the `.date` field of an `epoch`, to determine
the encountered timestamps:

```rust
   let epochs: Vec<_> = record
    .keys() // keys interator
    .map(|k| k.date) // building a key.date vector
    .collect();
```

according to `hashmap` documentation: `.keys()` are randomly exposed (not sorted):

```rust
epochs = [ // example
    2021-01-01T14:00:00,
    2021-01-01T10:00:00,
    2021-01-01T05:00:00,
    2021-01-01T22:00:00,
    ...
]
```

You can use `itertools` to enhance the iterator methods and sort them easily:

```rust
use itertools::Itertools;
let epochs: Vec<_> = record
    .keys()
    .sort() // unlocked
    .collect();

epochs = [ // example
    2021-01-01T00:00:00,
    2021-01-01T01:00:00,
    2021-01-01T03:59:44,
    2021-01-01T04:00:00,
    2021-01-01T05:00:00,
    ...
]
```

`unique()` filter is not available to hashmaps,
due to the `hashmap.insert()` behavior which always overwrites
a previous value for a given key.   
It is not needed in our case because, because `epochs` are unique,
we will always have a single set of data per epoch.

Keep in mind `epochs` are fixed to `EpochFlag::Ok` in NAV files.   
EpochFlags are provided in OBS files for examples.   
Refer to specific
[Observation files documentation](https://github.com/gwbres/rinex/blob/main/doc/observation.md)
to see how filtering using epoch flags is powerful and relevant.

## `Sv` object

`Sv` for Satellite Vehicule, is also 
used to sort and idenfity datasets.  
For instance, in a NAV file,
we have one set of NAV data per `Sv` per `epoch`.

`Sv` is tied to a `rinex::constellation` and comprises an 8 bit
identification number.

## Navigation Data

[Navigation Data documentation](https://github.com/gwbres/rinex/blob/main/doc/navigation.md)

## Observation Data

[Observation Data documentation](https://github.com/gwbres/rinex/blob/main/doc/observation.md)

## Meteo Data

[Meteo Data documentation](https://github.com/gwbres/rinex/blob/main/doc/meteo.md)

## Contribute

[How to contribute](https://github.com/gwbres/rinex/blob/main/doc/contribute.md)
