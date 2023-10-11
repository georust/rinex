# GNSS

[![crates.io](https://img.shields.io/crates/v/gnss-rs.svg)](https://crates.io/crates/gnss-rs)
[![crates.io](https://docs.rs/gnss-rs/badge.svg)](https://docs.rs/gnss-rs/badge.svg)

GNSS Constellations and Space Vehicles (SV) support in Rust

## Getting started

Add "gnss" to your Cargo.toml

```toml
gnss-rs = "2"
```

Import "gnss-rs": 

```rust
extern crate gnss_rs as gnss;
```

## Space Vehicles

```rust
extern crate gnss_rs as gnss;

use hifitime::TimeScale;
use gnss::sv;
use gnss::prelude::*;
use std::str::FromStr;

let sv = SV::new(Constellation::GPS, 1);
assert_eq!(sv.constellation, Constellation::GPS);
assert_eq!(sv.prn, 1);
assert_eq!(sv.timescale(), Some(TimeScale::GPST));
assert_eq!(sv, sv!("G01"));
assert_eq!(sv.launched_date(), None);
```

## SBAS support

We support SBAS (geostationary augmentations) systems. 

```rust
extern crate gnss_rs as gnss;

use gnss::sv;
use gnss::prelude::*;
use std::str::FromStr;
use hifitime::{Epoch, TimeScale};

let sv = sv!("S23");
assert_eq!(sv.constellation, Constellation::EGNOS);
let launched_date = Epoch::from_str("2021-11-01T00:00:00 UTC")
    .unwrap();
assert_eq!(sv.launched_date(), Some(launched_date));
```

## License

Licensed under either of:

* Apache Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
* MIT ([LICENSE-MIT](http://opensource.org/licenses/MIT)
