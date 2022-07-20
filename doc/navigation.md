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

Refer to this example to learn how to browse NAV `records`, that includes:
* [x] grab high level information
* [x] enumerate epochs contained in record
* [x] enumerate epochs for specific `Sv` contained in record
* [x] retrieve specific data, for specific `Sv` and specific `epochs`..

## Navigation Record content

The NAV record is sorted by `Epoch` and by `Sv`.   

`Epoch.flag` is not contained in a NAV file, as opposed to Observation Data.
Therefore, all _epochs_ contained in a NAV file are supposed to be sane / valid
and have a constant `epoch.flag::Ok` value. That also means 
filtering epoch.flags makes no sense for a NAV Data.

All NAV record share the following attributes:

* "msg": NAV message type (str)
* "type": NAV message type (str)
* "svClockBias": `Sv` Clock Bias [s] (f32)
* "svClockDrift": `Sv` Clock Drift [s.s⁻¹] (f32)
* "svClockDriftRate": `Sv` Clock Drift Rate [s.s⁻²] (f32)

All remaining NAV record content is GNSS & revision dependent and is described in 
_db/navigation.json_ descriptor. 

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

Every single data contained in a NAV record is wrapped in a `ComplexEnum` enum.   
Even the five first ones that are not constellation and revision dependent.   
`ComplexEnum` wrapper allows flexible and efficient parsing, of all
NAV revisions and type descriptions.

* "f32": unscaled float value
* "f64": unscaled double precision
* "str": string value
* "u8": raw 8 bit value

For example, to unwrap the actual data of a ComplexEnum::F32,
one should call: `as_f32().unwrap()`.
