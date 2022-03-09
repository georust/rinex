# RINEX Navigation Data

Navigation (NAV) `RINEX` is amongst the two most standard `RINEX` formats.  

There are two types of NAV records :
* `Ephemeride` (EPH) - most common
* `Ionospheric` (ION) 

NAV record type description was introduced in `RINEX > 3`,
therefore `type` is fixed to `EPH` for `header.version <= 3`

NAV message type description was also introduced in `RINEX > 3`,
`msg` is fixed to `LNAV` for `header.version <= 3`.

There are several types of NAV messages:

* `LNAV`   legacy &  `RINEX < 3`
* `CNAV`   civilian nav
* `CNAV2`  civilian nav
* `INAV`   integrity nav message
* `FNAV`   nav message
* `FDMA`  

One example script is provided
```bash
cargo run --example navigation
```

## Navigation Record content

The NAV record is sorted by `Epoch` and by `Sv`.   
All NAV record share the following attributes:

* "msg": NAV message type
* "type": NAV message type
* "svClockBias": `Sv` Clock Bias (s) - refer to RINEX definitions
* "svClockDrift": `Sv` Clock Drift (s.s⁻¹) - refer to RINEX definitions
* "svClockDriftRate": `Sv` Clock Drift Rate (s.s⁻²) - refer to RINEX definitions

All remaining NAV record content is GNSS & revision dependent and is described in 
_navigation.json_ descriptor. 

All keys described by this file descriptor are exposed. Refer to the official RINEX definition
of the keys you are interested in:

```json
[{
   "constellation": "GPS", << GPS (NAV) descriptors
   "revisions": [{ << revisions descriptor
       "revision": {
           "major": 1 << 1.0
       },
       "content": { << data fields
           "iode": "f32",
           "crs": "f32",
          "deltaN": "f32",
          "m0": "f32",
          "cuc": "f32",
          "e": "f32",
          "sqrta": "f32",
          "toe": "f32",
          "cic": "f32",
          "omage0": "f32",
          "cis": "f32",
          "i0": "f32",
          "crc": "f32",
          "omega": "f32",
          "omegaDot": "f32",
          "idot": "f32",
          "l2Codes": "f32",
          "gpsWeek": "f32",
          "l2pDataFlag": "f32",
          "svAccuracy": "f32",
          "svHealth": "f32",
          "tgd": "f32",
          "iodc": "f32",
          "cuc": "f32",
          "iodc": "f32",
          "t_tm": "f32",
          "fitInt": "f32"
    }
},
{
    "revision": { << revision 2
        "major": 2
    },
    "content": {
        "iode": "f32",
        "crs": "f32",
```

Item labelization type definition follow RINEX specifications closely.

## Navigation Record analysis

### High level `hashmap` indexing

`hashmap` allows high level indexing, it is possible to 
grab an epoch directly :

```rust
// match a specific `epoch`
let to_match = rinex::epoch::from_string(
   "21 01 01 09 00 00")
      .unwrap();
//    ---> retrieve all data for desired `epoch`
let matched_epoch = &record[&to_match];
```

Zoom in on `B07` vehicule (Beidou constellation, vehicule #7)
from that particular _epoch_ by indexing the inner hashmap directly:

```rust
// zoom in on `B07`
let to_match = rinex::record::Sv::new(
    rinex::constellation::Constellation::Beidou,
    0x07);
let matched_b07 = &matched_epoch[&to_match];
```

Zoom in on `clockDrift` field for that particular vehicule,
in that particular epoch using direct indexing:

```rust
let clkDrift = &matched_b07["ClockDrift"];
```

## Advanced manipulation using iterators & filters 

Extract all `E04` vehicule data from record,
by using the `Sv` object comparison method

```rust
let to_match = rinex::record::Sv::new(
    rinex::constellation::Constellation::Galileo,
    0x04);
let matched : Vec<_> = record
    .iter() // epoch iterator
    .map(|(_, sv)|{  // all epochs, about to filter sv
        sv.iter() // sv iterator
            .find(|(&sv, _)| sv == to_match) // comparison
    })
    .flatten() // dump non matching data
    .collect();
```

Extract `clockbias` and `clockdrift` fields
for `E04` vehicule accross entire record

```rust
let to_match = rinex::record::Sv::new(
    rinex::constellation::Constellation::Galileo,
    0x04);
let matched : Vec<_> = record
    .iter()
    .map(|(_, sv)|{
        sv.iter()
            .find(|(&sv, _)| sv == to_match)  // comparison
            .map(|(_, data)| ( // build a tuple
               data["ClockBias"] // field of interest
                  .as_f32() // unwrap actual value
                  .unwrap(),
               data["ClockDrift"]
                  .as_f32()
                  .unwrap(),
             ))
    })
    .flatten() // dump non matching data
    .collect();
```

Extract all data tied to `Galileo` constellation.
This time we only match one field of the `Sv` object
```rust
let to_match = rinex::constellation::Constellation::Galileo;
let matched : Vec<_> = record
    .iter() // epoch iterator
    .map(|(_, sv)|{  // epoch is left out, we filter on sv
        sv.iter() // sv iterator
            .find(|(&sv, _)| sv.constellation == to_match)  // sv.constellation field comparison
    })
    .flatten() // dump non matching data
    .collect();
```

Decimate `GPS` constellation data using a specific `epoch` interval
```rust
wip
```
