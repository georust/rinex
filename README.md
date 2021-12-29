# RINEX 
Rust package to parse and analyze Rinex files

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)
[![codecov](https://codecov.io/gh/gwbres/rinex/branch/main/graph/badge.svg)](https://codecov.io/gh/gwbres/rinex)

## Getting started

Many RINEX file types exist, 
the `RinexType` enum (refer to API) 
describes the types of RINEX currently supported:

* RinexType::NavigationMessage: Ephemeride NAV messages
* RinexType::ObservationData:   Observations

RINEX files contain a lot of information,
and this library exposes all data contained by supported
`Rinex Types`, that is: a lot.  

To determine how to operate this library, refer to:
* basic examples provided in this page
* basic examples delivered with the library (see down below)
* basic autotest method delivered by _src/lib.rs_ 

```shell
cargo run --example basic
```

If you're using this library, you probably want more
than that and actually retrieve your data of interest
and manipulate it.
To learn how to do this, refer to:
* more advanced examples described in this page:
 * basic data parsing, identification, extraction..
 * basic data sorting, 
 * data plotting..
* advanced examples delivered with the library (see down below)
* specific autotest methods delviered by _src/lib.rs_ 

```shell
cargo run --example nav-1
cargo run --example nav-2
```

### Supported RINEX revisions

* 2.00 ⩽ v < 4.0    Tested 
*             v < 2.0    should work
*             v = 4.0    should work, not garanteed

## Parsing a RINEX 

The __::from\_file__ method is how to parse a local ̀`RINEX` file: 

```rust
let rinex = Rinex::from_file("data/amel0010.21g");
println!("{:#?}", rinex);
```

```rust
let rinex = Rinex::from_file("data/aopr0010.17o");
println!("{:#?}", rinex);
```

### RINEX Header
The RINEX Header contains
All "Comments" are currently discarded and not treated.   

`length` of a Rinex file returns the number of payload items:

+ Total number of Observations for `RinexType::ObservationData`
+ Total number of Nav Messages for `RinexType::NavigationMessage`

```rust
let rinex = Rinex::from_file("data/amel0010.21g");
println!("{}", rinex.len());
```

## RINEX Header

The `Rinex Header` inner object contains all
general and high level information 

```rust
let rinex = Rinex::from_file("data/AMEL00NLD_R_20210010000_01D_MN.rnx");
let header = rinex.get_header();
println!("{:#?}", header);
assert_eq!(header.get_type(), RinexType::NavigationMessage);
assert_eq!(header.get_constellation(), constellation::Constellation::Mixed);
```

Other example

```rust
let rinex = Rinex::from_file("data/dlf10010.21g");
let header = rinex.get_header();
println!("{:#?}", header);
assert_eq!(header.get_version().to_string(), "2.11"); 
assert_eq!(header.get_pgm(), "teqc  2019Feb25");
assert_eq!(header.run_by(),  "Unknown");
```

### GPS Observation example
TODO

## RINEX File Types

### RINEX: Navigation Message (NAV)

Three main constellations are currently supported 

+ GPS
+ Glonass
+ Galileo
+ Mixed (GPS,Glonass,Galileo)

### RINEX: Observation Data (OBS)
TODO

## Data indexing interface

This lib builds a dictionnary interface to interact, sort and retrieve
RINEX files payloads. Some restrictions may apply to certains GNSS constellations,
refer to specific paragraphs down below.

All known keys are referenced in the /keys folder.  
For key and related data specifications, refer toe the related "DATA RECORD" description
in the `RINEX` specifications


### GPS Navigation analysis example
TODO

### Glonass Navigation analysis example  
TODO

### Mixed Navigation example
TODO

### Mixed Observation example
TODO

## Supporting new RINEX Revisions

For all currently supported `RinexTypes` 
1: go through all possible record modifications:
 * new types
 * new values..
2: add a new entry to keys.json
3: add some new test resources 
4: add new specific test method
