RINEX crate
===========

[![crates.io](https://img.shields.io/crates/v/rinex.svg)](https://crates.io/crates/rinex)
[![rustc](https://img.shields.io/badge/rustc-1.61%2B-orange.svg)](https://img.shields.io/badge/rustc-1.61%2B-orange.svg)

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex) 
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT)

## Parser

`RINEX` files contain a lot of data and this library is capable of parsing most of it.   
To fully understand how to operate this lib, refer to the `RinexType` section you are interested in.

## Production (writer)

`RINEX` file production (writer) is work in progress and will be supported
in next releases

## File naming convention

This parser does not care for the RINEX file name, it is possible to parse a file    
that does not respect standard naming conventions.

### Known weaknesses

* Unusual RINEX files with a unique epoch may not be parsed correctly in some cases
* Only Observation Data V > 2 currently has a format restriction, see dedicated page

## Getting started 

The ``Rinex::from_file`` method parses a local `RINEX` file:

```rust
let rinex = rinex::Rinex::from_file("test_resources/NAV/V2/amel0010.21g")
  .unwrap();
```

The `test_resources/` folder contains short but relevant RINEX files, 
spanning almost all revisions and supported types, mainly for CI purposes.

For data analysis and manipulation, you must refer to the
[official RINEX definition](https://files.igs.org/pub/data/format/)

This [interactive portal](https://gage.upc.edu/gFD/) 
is also a nice interface to discover `RINEX`. 

### Header & general information

The `header` contains high level information.   

```rust
println!("{:#?}", rinex.header);
```

This includes `Rinex`:
* revision number
* GNSS constellation
* possible file compression infos
* recorder & station infos
* hardware, RF infos
* `comments` are exposed in a string array, by order of appearance 
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

Most record content are sorted by epochs, but that is RINEX dependent:
refer to the main page for more information. When a record
is indexed by epochs, that means it is sorted by sampling timestamps
and an `epoch::Flag` validating this epoch.
Note that epochs Flags are only relevant in Observation Data;
it is fixed to "Ok" in other records.
RINEX files usually span 24h at a steady sampling interval.   

`record` is a complex structure, which depends 
on the RINEX type. In this paragraph, we expose how to iterate (browse)
every supported record types. Advanced users
must differentiate between Vector inner data and Map (Hash or BTree) inner data.
The only difference is basically how you reference the internal data:
* Vector: by position index (integer)
* Map (hash or btree): by an object. This provides efficient data classification right away.

The difference between a Hash and a BTree map, is that the btreemap
is naturally sorted. This is the type we use anytime we need to guarantee classification
at all times. For example, in epochs so the record is chronologically sorted.

* Navigation Record browsing

```rust
let record = record.as_nav()
    .unwrap(); // user must verify this is feasible
for (epoch, classes) in record.iter() { // Navigation frame classes, per epoch
    for (class, frames) in classes.iter() { // Per frame class
        for frame in frames.iter() { // all frames for this epoch and this kind of frame
          // several frame classes exist
          if *class = navigation::record::FrameClass::Ephemeris {
              // Ephemeris are the most common NAV frames
              // Until V < 4, their the only ones provided.
              let (msgtype, sv, clk, clk_dr, clk_drr, map) = frame.as_eph() // Unwrap as Ephemeris
                .unwrap(); // you're fine, thanks to the previous == check
              
              // several MsgTypes exist, 
              // up until V < 4, they are marked as Legacy NAV whatever happens.
              // Modern NAV can also contain Legacy frames.
              assert_eq!(msgtype, navigation::record::MsgType::LNAV);

              // All ephemeris contain a satellite vehicule,
              // its internal clock bias [s], clock drift [s/s] and clock drift rate [s/s^2]
              assert_eq!(sv.constellation, Constellation::GPS);
              assert_eq!(clk, 1.0);
              asssert_eq!(clk_dr, 2.0);
              asssert_eq!(clk_drr, 3.0);

              // Remaining data is integrated to a complex map,
              // as it depends on the File Revision & the current Constellation.
              // Index keys can be found in the db/NAV/navigation.json descriptor,
              // it follows RINEX specifications. 
              // All data is currently interprated a floating point double precision (f64),
              // other interpratation like Binary Flags remain to do to this day
              let iode = map["iode"].as_f64();
              assert_eq!(iode, 12345.0);
              let satPosX = map["satPosX"].as_f64();
              assert_eq!(satPosX, 5678.0);
          
          } else if *class == navigation::record::FrameClass::IonosphericMdodel {
            // Ionospheric models can be found in Modern NAV RINEX.
            let model = frame.as_ion() // unwrap as Ionospheric Model
              .unwrap(); // you're fine, thanks to the previous == check
              
            // Several Ionospheric models exist,
            // refer to RINEX specifications to understand the inner data and their units
            if let Some(model) = model.as_klobuchar() {
              assert_eq!(model.alpha.0, 0.0);
              assert_eq!(model.beta.3, 1.0);
              assert_eq!(model.region, navigation::ionmessage::KbRegionCode::WideArea);
              
            } else if let Some(model) = model.as_nequick_g() {
              assert_eq!(model.a.0, 0.0);
              assert_eq!(model.a.1, 1.0);
              // NG model region is a rust bitflag object, which allows convenient bit masking
              assert_eq!(model.region, navigation::ionmessage::NgRegionFlags::REGION5);
              assert_eq!(model.region.intersects(navigation::ionmessage::NgRegionlags::REGION1), true); // AND mask

            } else if let Some(model) = model.as_bdgim() {
              assert_eq!(model.alpha.0, 0.0);
              assert_eq!(model.alpha.1, 1.0);
            }
          
          } else if *class == navigation::record::FrameClass::SystemTimeOffset {
            let sto = frame.as_sto() // unwrap as STO message
              .unwrap(); // you're fine, thanks to the previous == check
            assert_eq!(sto.system, "GPUT"); // Time System descriptor
            assert_eq!(a.0, 1.0);
            assert_eq!(a.1, 2.0);
            assert_eq!(a.t_tm, 10000); // GNSS week counter
          }
        }
    }
}
```

* Observation Record browsing

```rust
let record = record.as_obs()
    .unwrap(); // user must verify this is feasible
for (epoch, (clk, sv)) in record.iter() { // Complex structures, on an `Epoch` basis
    // clk : is an optionnal (f32) clock offset for this epoch,
    for (sv, data) in sv.iter() { // Complex struct, on a `Space Vehicule` basis
        for (code, data) in data.iter() {
           // code is an observable: use this to determine the physics measured
           // data is an ObservationData,
           // which comprises the raw data, and possible an LLI flag and an SSI value
        }
    }
}
```

* Meteo Record browsing

```rust
let record = record.as_nav()
    .unwrap(); // user must verify this is feasible
for (epoch, data) in record.iter() { // Complex structures, on an `Epoch` basis
    for (obs, data) in data.iter() { 
        // obs: is a Meteo Observable: determines the physics measured
        // data: f32 raw data
    }
}
```

* Antex Record browsing

```rust
let record = record.as_antex()
  .unwrap(); // user must verify this is feasible
for (antenna, frequencies) in record.iter() {
  // several calibration methods exist
  assert_eq!(antenna.calibration.method, antex::record::Method::Chamber);
  assert_eq!(antenna.calibration.agency, "Some Agency");
  assert_eq!(antenna.calibration.date, "Some DateTime description");
  assert_eq!(antenna.sn, "Some Serial Number");
  assert_eq!(antenna.dazi, 1.0);
  assert_eq!(antenna.valid_from, Some(chrono::NaiveDateTime));
  assert_eq!(antenna.valid_until, Some(chrono::NaiveDateTime));
  for frequency in frequencies.iter() {
    assert_eq!(frequency.channel, channel::Channel::L1);
    assert_eq!(frequency.north, 10.0);
    assert_eq!(frequency.up, 20.0);
    for pattern in frequency.patterns {
      assert_eq!(pattern.is_azimuth_dependent(), true);
      let Some((azimuth, phase_pattern)) = pattern.azimuth_pattern() {
        for raw_phase in phase_pattern.iter() {
        
        }
      }
    }
  }
}
```

## `Epoch` object

[Epoch structure](https://docs.rs/rinex/latest/rinex/epoch/index.html)

`epoch` is a `chrono::NaiveDateTime` object validated by an
`EpochFlag`.    
A valid epoch is validated with `EpochFlag::Ok`, refer to specific API.

To demonstrate how to operate the `epoch` API, we'll take 
a Navigation Rinex file as an example. 

First, let's grab the record:

```rust
let rinex = rinex::Rinex::from_file("test_resource/NAV/V2/amel0010.21g")
  .unwrap();
let record = rinex.record
    .as_nav() // NAV record unwrapping
    .unwrap(); // would fail on other RINEX types
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

epochs = [ // epochs are sorted by timestamps 
    2021-01-01T00:00:00,
    2021-01-01T01:00:00,
    2021-01-01T03:59:44,
    2021-01-01T04:00:00,
    2021-01-01T05:00:00,
    ...
]
```

`unique()` to filter `epochs` makes no sense - and is not available,
because a valid RINEX exposes a unique data set per `epoch`.   

For RINEX files that do not expose an Epoch `flag`, like Navigation or Meteo data,
we fix it to `Ok` (valid `epoch`) by default.

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

## Useful high level methods

* `interval`: returns the nominal sampling interval - nominal time difference
between two successive epochs in this `record`. See API for more information
* `sampling_dead_time`: returns a list of `epochs` for which time difference
between epoch and previous epoch exceeded the nominal sampling interval.

### Advanced `RINEX` file operations

Advanced operations like file Merging will be available in next releases.   
Merged `RINEX` files will be parsed properly: meaning the `record` is built correctly.   
Informations on the `merging` operation will be exposed either in the `.comments` structure,
if "standard" comments were provided.

## Specific documentation

Documentation and example of use for specific `RINEX` formats 

+ [Navigation Data](https://github.com/gwbres/rinex/blob/main/doc/navigation.md)
+ [Observation Data](https://github.com/gwbres/rinex/blob/main/doc/observation.md)
+ [Meteo Data](https://github.com/gwbres/rinex/blob/main/doc/meteo.md)

## Work in progress

Topics to be unlocked by next releases

* RINEX file production : provide `to_file` methods
to produce supported RINEX formats
* RINEX Clock data type
* RINEX special operations like `merging` when producing a new file
* Merging + compression for OBS data

## Contribute

[How to contribute](https://github.com/gwbres/rinex/blob/main/doc/contribute.md)
