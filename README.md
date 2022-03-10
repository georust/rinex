# RINEX 
Rust package to parse and analyze Rinex files

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)
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

## Getting started 

Use ``Rinex::from_file`` to parse a local `RINEX` file:

```rust
let path = std::path::PathBuf::from("amel0010.21g");
let rinex = rinex::Rinex::from_file(&path).unwrap();
```

The `data/` folder contains a bunch of `RINEX` files, spanning almost all revisions
and all supported file types, mainly
for CI purposes: you can refer to them.

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

The `Rinex` structure comprises the header previously defined,
and the `Record` which contains the data payload.

The `Record` is optionnal at the moment and 
set to _None_ in case of CRINEX Observation Data,
as this lib is not able to decompress the file content. 
In that scenario, only the header is gets parsed.
Until this lib gets enhanced, you need to manually decompress
your compressed OBS files prior using this parser, if you want
your OBS file to be fully processed.

The `Record` definition varies with the type of Rinex file,
refer to its definition in the API and the specific documentation down below,
for the type of file you are interested in.

`Record` is a complex structure of HashMap (_dictionaries_),
which definition may vary. The first indexing at the moment, is always an `epoch`,
that is, data is sorted by the sampling timestamp.

## `Epoch` object

[Epoch structure](https://docs.rs/rinex/latest/rinex/epoch/index.html)

`epoch` is a `chrono::NaiveDateTime` object validated by an
`EpochFlag`.

To demonstrate how to operate the `epoch` API, we'll take 
a Navigation Rinex file as an example. First, grab the record:

```rust
let rinex = rinex::Rinex::from_file("navigation-file.rnx")
  .unwrap();
let record = rinex.record
  .unwrap() // option<record>, None in case of CRINEX
    .as_nav() // NAV record example
    .unwrap();
```

`epochs` serve as keys of the first hashmap. 
The `keys()` iterator is then the easiest way to to determine
which _epochs_ were idenfitied:

```rust
   let epochs: Vec<_> = record
    .keys() // keys interator
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

## `Sv` object

`Sv` for Satellite Vehicule, is also 
used to sort and idenfity datasets.  
For instance, in a NAV file,
we have one set of NAV data per `Sv` per `epoch`.

`Sv` is tied to a `rinex::constellation` and comprises an 8 bit
identification number.

## Data payload

The final layer of a record is what we are interested in.

The data payload are described in the specific documentation pages down below,
they vary for each RINEX file types.

In any case, data is encapsulated in the `ComplexEnum` enum,
which wraps:

* "f32": unscaled float value
* "f64": unscaled double precision
* "str": string value
* "u8": raw 8 bit value

For example, to unwrap the actual data of a ComplexEnum::F32,
one should call: `as_f32().unwrap()`.

Refer to the `ComplexEnum` API and following detailed examples.

## Navigation Data

Refer to related API and
[Navigation Data documentation](https://github.com/gwbres/rinex/blob/main/doc/navigation.md)

## Observation Data

Refer to related API and
[Observation Data documentation](https://github.com/gwbres/rinex/blob/main/doc/observation.md)

## GNSS Time specific operations

wip

## Meteo Data

wip

## Clocks data

wip
