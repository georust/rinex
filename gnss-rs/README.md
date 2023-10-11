# GNSS

[![crates.io](https://img.shields.io/crates/v/gnss-rs.svg)](https://crates.io/crates/gnss-rs)
[![crates.io](https://docs.rs/gnss-rs/badge.svg)](https://docs.rs/gnss-rs/badge.svg)

GNSS Constellations and Space Vehicles (SV) support in Rust

## Getting started

```rust
use hifitime::TimeScale;
extern crate gnss_rs as gnss;

let sv = SV::from_str("G23");
assert_eq!(sv, sv!("G23"));
assert_eq!(sv.constellation, Constellation::GPS);
assert_eq!(sv.timescale(), Some(TimesScale::GPST));
```

## SBAS support

We support SBAS (geostationary augmentations) systems. 

```rust
use hifitime::TimeScale;
extern crate gnss_rs as gnss;

let sv = SV::from_str("S23");
assert_eq!(sv, sv!("S23"));
assert_eq!(sv.constellation, Constellation::EGNOS);
```

## License

Licensed under either of:

* Apache Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
* MIT ([LICENSE-MIT](http://opensource.org/licenses/MIT)
