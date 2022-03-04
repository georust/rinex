# RINEX:: Navigation Data

Navigation (NAV) `RINEX` is amongst the two most standard `RINEX` formats.  

There are two types of NAV records :
* `Ephemeride` (EPH) - most common
* `Ionospheric` (ION) 

NAV record type description was introduced in `RINEX > 3`,
therefore `type` is fixed to `EPH` for `header.version <= 3`

NAV message type description was also introduced in `RINEX > 3`,
`msg` is fixed to `LNAV` for `header.version <= 3`.

There are several types of NAV message:

* `LNAV`  : legacy &  `RINEX < 3`
* `CNAV`  : civilian nav
* `CNAV2` : civilian nav
* `INAV`  : integrity nav message
* `FNAV`  : nav message
* `FDMA`  :

## Navigation Record content

The Record content depends on the GNSS constellation (Mixed or Unique)
and Rinex revision.

All NAV record are stored by `Epoch` and by `Sv` (sat. vehicule).   
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
navigation.json
    "NavigationMessage": { <<-- rinex type
      "LNVA": <-- a new category should be introduced
                  to describe V ≥ 4 correctly
        "GPS": { <<-- constellation
            format is "id": "type",
            // this one describes how item1 is identified
            "iode": "d19.12", // follow RINEX specifications
            "crs": "d19.12", // item2
            // [..]
            "idot": "d19.12", // item10
            "spare": "xxxx", // orbit empty field, 
                           // but continue parsing..
            "tgd": "d19.12" // item11
            // parsing stops there
        }
    }
```

Item labelization type definition follow RINX specifications closely.

Item types: 

* "d19.12": ends up as float value (unscaled)
* "iode": key example - as described in RINEX tables
* "crs": key example - as described in RINEX tables


## Navigation Record analysis

All records are sorted by observation `Epoch`

### List all Epochs

```rust
let epochs = rinex.record.iter()
    .unique()
    .collect(); 
println!("{:#?}", epochs);
```
`unique()` to filter unique `Epoch` values is not really needed here,
since a sane NAV Rinex will have a single realization per epoch,
for a specific satellite vehicule.

### Determine all encountered satellite vehicules

Accross all epochs, we filter unique `Sv`

### Retain all data of specific `Sv`

Across all epochs we retain everything matching a specific `Sv`

### Retain all data tied to Glonass Constellation

Match a specific constellation amongst all epochs:

### Extract `Sv` clock bias and clock drift for `G01`

Extract specific data for a specific satellite vehicule

### Extract `Sv` clock bias sampled at a specific `Epoch`

Epoch filtering:

### Decimate `Sv` clock bias for a specific `Epoch` /sampling interval

Decimating data using `Epoch` field:

