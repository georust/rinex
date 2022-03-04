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

Link to the [official API](https://docs.rs/rinex/0.0.10/rinex/)

### Supported RINEX revisions

* 2.00 ⩽ v < 4.0    Tested 
*             v = 4.0    should work, not garanteed at the moment

## Getting started 

Use ``Rinex::from_file`` to parse a local `RINEX` file:

```rust
let path = std::path::PathBuf::from("amel0010.21g");
let rinex = Rinex::from_file(&path).unwrap();
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
* physical, RF and other infos

Once again, refer to complete API

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

## Navigation Data

Refer to related API and
[Navigation Data documentation](https://github.com/gwbres/rinex/blob/main/doc/navigation.md)

## Observation Data

Refer to related API and
[Observation Data documentation](https://github.com/gwbres/rinex/blob/main/doc/observation.md)

## GNSS Time

wip

## Meteo Data

wip

## Clocks data

wip
