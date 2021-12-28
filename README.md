# RINEX 
Rust package to parse and analyze Rinex files

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)
[![codecov](https://codecov.io/gh/gwbres/rinex/branch/main/graph/badge.svg)](https://codecov.io/gh/gwbres/rinex)

## Getting started

Use ::from_file to parse a RINEX file:

```rust
let rinex = Rinex::from_file("data/amel0010.21g");
println!("{:#?}", rinex);

let rinex = Rinex::from_file("data/aopr0010.17o");
println!("{:#?}", rinex);
```

All "Comments" are currently discarded and not treated.   

`length` of a Rinex file returns the number of payload items:

+ Total number of Observations for `RinexType::ObservationData`
+ Total number of Nav Messages for `RinexType::NavigationMessage`

```rust
let rinex = Rinex::from_file("data/amel0010.21g");
println!("{}", rinex.len());
```

### Supported versions

2.04 <= v <= 3.11 has been extensively tested and should work on most RINEX files.   
Refer to /data to see the database against which this lib is automatically tested.

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

## Data interface

This lib builds a dictionnary interface to interact, sort and retrieve
RINEX files payloads. Some restrictions may apply to certains GNSS constellations,
refer to specific paragraphs down below.

To determine the key of interest, refer to the specific RINEX payload documentation.
The key will be the "official" denomination without white space/underscore. 
Some restrictions will apply for complex key names, they will be explained in the related
sections down below.

Here is an example of the labelization used for GPS Navigation Message payload

### GPS Navigation example
TODO

### Glonass Navigation example  
TODO

### Mixed Navigation example
TODO

### Mixed Observation example
TODO

### GPS Observation example
TODO

## RINEX Types

Many RINEX file types exists, `RinexType` (refer to API) describes some of them.  
The payload of a RINEX file depends on the Rinex type.  
For each supported type this library provides a convenient interface to 
manipulate the file content.

### Navigation Message data

Navigation Messages are Constellation dependent. Three main constellations
are currently supported 

+ GPS
+ Glonass
+ Galileo
+ Mixed (GPS,Glonass,Galileo)

That means the lib will not build internal data against other unique GNSS constellation files.

### Observation data
TODO
