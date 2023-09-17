# SP3

[![crates.io](https://img.shields.io/crates/v/sp3.svg)](https://crates.io/crates/sp3)
[![Rust](https://github.com/georust/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/sp3/badge.svg)](https://docs.rs/sp3/)
[![crates.io](https://img.shields.io/crates/d/sp3.svg)](https://crates.io/crates/sp3)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/georust/rinex/sp3/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/georust/rinex/sp3/blob/main/LICENSE-MIT) 

SP3 Precise GNSS Orbit files parser. 

SP3 is specifid by [IGS](https://igs.org/products/#orbits_clocks).

The parser only supports Revisions C & D at the moment and rejects revisions A & B.

## Getting started

Add "sp3" to you cargo file

```toml
[dependencies]
sp3 = "1"
```

Parse an SP3 file

```rust
use crate::prelude::*;
use rinex::prelude::Constellation;
use std::path::PathBuf;
use std::str::FromStr;
    
let path = PathBuf::new()
    .join(env!("CARGO_MANIFEST_DIR"))
    .join("data")
    .join("ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz");

let sp3 = SP3::from_file(&path.to_string_lossy());
assert!(
    sp3.is_ok(),
    "failed to parse ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz : {:?}",
    sp3.err()
);

let sp3 = sp3.unwrap();

/*
 * Test general infos
 */
assert_eq!(sp3.version, Version::C);
assert_eq!(sp3.data_type, DataType::Position);

assert_eq!(
    sp3.first_epoch(),
    Some(Epoch::from_str("2023-08-27T00:00:00 GPST").unwrap())
);

assert_eq!(sp3.nb_epochs(), 96, "bad number of epochs");
assert_eq!(sp3.coord_system, "ITRF2");
assert_eq!(sp3.orbit_type, OrbitType::BHN);
assert_eq!(sp3.time_system, TimeScale::GPST);
assert_eq!(sp3.constellation, Constellation::Mixed);
assert_eq!(sp3.agency, "ESOC");

assert_eq!(sp3.week_counter, (2277, 0.0_f64));
assert_eq!(sp3.epoch_interval, Duration::from_seconds(900.0_f64));

// browse SV positions
for (epoch, sv, (x, y, z)) in sp3.sv_position() {

}

// browse SV clock
for (epoch, sv, clock) in sp3.sv_clock() {

}
```

## File Merge

Merge files together, for example to create a context spanning 48 hours

```rust
let folder = PathBuf::new()
    .join(env!("CARGO_MANIFEST_DIR"))
    .join("data");

let sp3_a = folder.clone()
    .join("ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz");

let sp3_b = folder.clone()
    .join("ESA0OPSULT_20232320600_02D_15M_ORB.SP3.gz");

let sp3 = SP3::from_file(&sp3_a.to_string_lossy())
    .unwrap();

let sp3_b = SP3::from_file(&sp3_b.to_string_lossy())
    .unwrap();

let sp3 = sp3_a.merge(sp3_b);
assert!(sp3.is_ok());
```

## Position Vector Interpolation

Interpolate SV position at desired Epoch.  
In order to preserve the high (+/- 1mm precision) for SP3 datasets,
we recommend using at least an interpolation order of 9 for typical SP3 files
with 15' epoch intervals.

The current implementation restricts the interpolatable Epochs at 
[tmin, tmax] = [(N +1)/2 * τ, T(n-1) - (N +1)/2 * τ],
both included, where N is the interpolation order, τ the epoch interval, and T(n-1)
the last Epoch in this file.

Refer to the online API for more information

```rust
use sp3::prelude::*;
use rinex::sv;
use std::str::FromStr;
use std::path::PathBuf;
use rinex::prelude::Sv;

let path = PathBuf::new()
    .join(env!("CARGO_MANIFEST_DIR"))
    .join("data")
    .join("ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz");

let sp3 = SP3::from_file(&path.to_string_lossy())
    .unwrap();

let epoch = Epoch::from_str("2023-08-27T00:00:00 GPST")
    .unwrap();
let interpolated = sp3.interpolate(epoch, sv!("G01"), 11);
assert!(interpolated.is_none(), "too early in this file");

let epoch = Epoch::from_str("2023-08-27T08:15:00 GPST")
   .unwrap();
let interpolated = sp3.interpolate(epoch, sv!("G01"), 11);
assert!(interpolated.is_some());
let (x, y, z) = interpolated.unwrap();
// demonstrate error is still sub cm
assert!((x - 13281.083885).abs() * 1.0E3 < 1.0E-2); // distances are expressed in km in all SP3
assert!((y - -11661.887057).abs() * 1.0E3 < 1.0E-2);
assert!((z - 19365.687261).abs() * 1.0E3 < 1.0E-2);
```
