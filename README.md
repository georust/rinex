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

To learn how to operate this library, refer to:
* the examples delivered with the library are detailed and compelling
* extracted examples on this page 
* autotest methods delivered by _src/lib.rs_ 

### Supported RINEX revisions

* 2.00 ⩽ v < 4.0    Tested 
*             v = 4.0    should work, not garanteed at the moment

## Running the examples

```shell
cargo run --example nav-simple
cargo run --example nav-mixed
```

* nav-simple: simple NAV file manipulation
* nav-mixed: simple NAV::Mixed file manipulation

## Parsing a RINEX 

The __::from\_file__ method lets you parse a local `RINEX` file: 

```rust
use rinex::*;
let path = std::path::PathBuf::from("amel0010.21g");
let rinex = Rinex::from_file(&path).unwrap();
```

### RINEX Header

The `Rinex Header` contains high level information

```rust
use rinex::*;
let rinex = Rinex::from_file(&PathBuf::from("amel0010.21g")).unwrap();
let header = rinex.get_header();
println!("{:#?}", header);
assert_eq!(header.get_type(), RinexType::NavigationMessage);
assert_eq!(header.get_constellation(), constellation::Constellation::Mixed);
println!("{:#?}", header.get_rinex_version());
println!("Record size: {}", rinex.len(); 
```

"Comments" are currently discarded and not exposed by the parser.   

`length` of a Rinex file returns the number of payload items:

+ Total number of Observations for `RinexType::ObservationData`
+ Total number of Nav Messages for `RinexType::NavigationMessage`

## RINEX Record and File types

The Record (file content) depends on the file type and constellation.   
To learn how to operate the library for the Rinex type you are interested in,
refer to the following section.

## Navigation Message (NAV)

NAV records expose the following keys to manipulate & search through the records:
* `sv` : Satellite Vehicule ID (_src/record.rs::Sv_)
* `svClockBias`: clock bias (s)
* `svClockDrift`: clock drift (s.s⁻¹)
* `svClockDriftRate`: clock drift (s.s⁻²)
* all keys contained in _keys.json_ for related Rinex revision & constellation

Two main cases for `RinexType::NavigationMessage`:
+ Unique constellation (e.g `Constellation::GPS`) 
+ `Constellation::Mixed` (modern use case?)

### NAV: determine encountered Sattelite Vehicules (`Sv`) 

```rust
    // 'example --nav-simple'
    let vehicules: Vec<_> = rinex.get_record().iter()
        .map(|s| s["sv"])
        .collect();
```

### NAV (mixed): extract all `Sv` tied to `Glonass`

```rust
    // 'example --nav-simple'
    let vehicules: Vec<_> = rinex.get_record().iter()
        .map(|s| s["sv"])
        .collect();

    let glo_vehicules: Vec<_> = vehicules.iter()
        .filter(|s| s.Sv().unwrap()
            .get_constellation() == Constellation::Glonass)
        .collecr();
```

### NAV (mixed): extract `svClockBias`(s) and `svClockDrift` (s.s⁻¹) for `R03`

```rust
    // 'example --nav-simple'
    let sv = Sv::new(Constellation::Glonass, 0x03); // `R03`
    let sv_to_match = RecordItem::Sv(sv); // filter item
    let r03_data: Vec<_> = rinex.get_record().iter()
        .map(|s| s["sv"] == sv_to_match)
        .collect();
    
    let (clk_bias, clk_drift) = (r03_data["svClockBias"], r03_data["svClockDrift"]);
```

### NAV (mixed): extract specific data
>>WorkInProgress
`svHealth` exists for GPS for instance (_keys.json_) and
is a binary word:

```rust
    // 'example --nav-mixed'
```

## Observation data (OBS)
>>WorkInProgress

OBS records expose mainly:
* raw carrier phase measurements
* pseudo ranges estimates
* doppler measurements

### OBS: determine which measurements we have
>>WorkInProgress

First thing to do is to determine which
data a given file actually contains

```rust
    // 'example --obs-simple'
    let obs_types = header.get_obs_types();
```

### OBS (GPS): extract pseudo range
>>WorkInProgress

Extract pseudo range for a unique vehicule

``̀`rust
    // 'example --obs-simple'
    let sv = Sv::new(Constellation::GPS, 0x01); // "G01"
    let to_match = RecordItem::Sv(sv); // filter item
    let matching: Vec<_> = rinex.get_record().iter()
        .filter(|s| s["sv"] == to_match)
        .collect();
    let (pseudo_range, raw_phase) = matching["C1C"];
```

<!> Sometimes data is scaled:  
TODO

```rust
    // 'example --obs-mixed'
```

### OBS: complex data extract on `SigStrength` condition
>>WorkInProgress

Extract phase + pseudo range

### Use `LeapSecond` information
>>WorkInProgress

```rust
    // 'example --nav-mixed'
```

### Ionosphere compensation 
>>WorkInProgress

```rust
    // 'example --nav-mixed'
```

### System time compensation
>>WorkInProgress

```rust
    // 'example --'
```
