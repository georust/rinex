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

* `RinexType::NavigationMessage`: Ephemeride (NAV) messages
* `RinexType::ObservationData`:   Observations (OBS) data

RINEX files contain a lot of information,
and this library exposes all data contained by supported
`Rinex Types`, that means: a lot.

To determine how to operate this library, refer to:
* examples provided in this page
* examples delivered with the library (see down below)
* autotest methods delivered by _src/lib.rs_ 

### Supported RINEX revisions

* 2.00 ⩽ v < 4.0    Tested 
*             v = 4.0    should work, not garanteed at the moment

## Running the examples

```shell
cargo run --example nav-simple
```

* basic: basic parser using header information
* nav-simple: simple NAV file manipulation

## Parsing a RINEX 

The __::from\_file__ method is how to parse a local `RINEX` file: 

```rust
let rinex = Rinex::from_file("data/amel0010.21g");
println!("{:#?}", rinex);
```

```rust
let rinex = Rinex::from_file("data/aopr0010.17o");
println!("{:#?}", rinex);
```

## RINEX Header

The `Rinex Header` contains high level information

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

"Comments" are currently discarded and not exposed by the parser.   

`length` of a Rinex file returns the number of payload items:

+ Total number of Observations for `RinexType::ObservationData`
+ Total number of Nav Messages for `RinexType::NavigationMessage`

```rust
let rinex = Rinex::from_file("data/amel0010.21g");
println!("{}", rinex.len());
```

## RINEX Record and File types

The Record (file content) depends on the file type and constellation.   
To learn how to operate the library for the Rinex type you are interested in,
refer to the following section.

### Navigation Message (NAV)

NAV records expose the following keys to manipulate & search through the records:
* "sv" : Satellite Vehicule ID (_src/record.rs::Sv_)
* "svClockBias": clock bias (s)
* "svClockDrift": clock drift (s.s⁻¹)
* "svClockDriftRate": clock drift (s.s⁻²)
* all keys contained in _keys.json_ for related Rinex revision & constellation

Two main cases for `RinexType::NavigationMessage`:
+ Unique constellation (e.g `Constellation::GPS`) 

```rust
    // extracted from 'example --nav-simple'
    use rinex;
    use rinex::record::*;
    use rinex::constellation::Constellation;

    let rinex = Rinex::from_file("data/amel0010.21g"); // GLONASS NAV <-> unique
    assert_eq!(rinex.header.get_constellation(), Constellation::Glonass);

    // each record were recorded against an "RXX" Sat. Vehicule
    let record = rinex.get_record(0); // first entry
    println!("{:#?}", record);

    // RXX filter is pointless
    let rxx_vehicules = rinex.match_filter(Constellation::Glonass);
    // R01 filter is convenient 
    let to_match = RinexRecordItem::Sv(Sv::new(Constellation::Glonass, 0x01));
    let records = rinex.match_filter(to_match);
```

+ `Constellation::Mixed` (modern use case?)

In this context, the constellation information must be retrieved
at the `RecordItem` level. A pre filter comes handy:

```rust
    // extracted from 'example --nav-mixed
    use rinex;
    use rinex::record::*;
    use rinex::constellation::Constellation;

    // MIXED / V3 Modern / 
    // MIXED GPS/GAL/GLO mainly - V3(modern)
    let rinex = Rinex::from_file("data/AMEL00NLD_R_20210010000_01D_MN.rnx"); 
    assert_eq(rinex.header.get_constellation(), Constellation::Glonass); // nope
    assert_eq!(rinex.header.get_constellation(), Constellation::Mixed);  // yes

    let record = rinex.get_record(0); // first entry
    println!("{:#?}", record);
    
    // GPS filter 
    let records = rinex.match_filter(Constellation::GPS);
    // G01 filter
    let to_match = RinexRecordItem::Sv(Sv::new(Constellation::GPS, 0x01));
    let records = gxx_records.match_filter(to_match);
```

### Observation Date (OBS)

TODO
