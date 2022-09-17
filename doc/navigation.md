Navigation Data
===============

Navigation (NAV) `RINEX` is amongst the two most standard `RINEX` formats.  

NAV record type description was introduced in `RINEX > 3`,
therefore `type` is fixed to `EPH` for `header.version <= 3`. 

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
a "database" description _db/navigation.json_. 

All keys described by this file descriptor are to be parsed. 
You can use the database descriptor to retrieve your data of interest.
Also, we try to have "keys" (dictionary identifiers) close to the RINEX definitions.

```json
[{
   "constellation": "GPS", # GPS descriptors
   "revisions": [{ # revisions descriptor
       "revision": {
           "major": 1 # 1.0
       },
       "content": { # data to be parsed for GPS1
          "iode": "f32", # evaluated as float 
          "crs": "f32",
          "deltaN": "f32",
		  [..]
          "t_tm": "f32",
          "fitInt": "f32"
    }
},
```

Parsed data is wrapped in "DbItem" data wrapper, which supports
the following native types:

* "str": string value
* "u8": unsigned 8 bit value
* "f32": float value
* "f64": double precision

To unwrap as "f32" for instance, 
user should invoke `as_f32()`.

DbItem currently does not support scaling. Parsed data is not rescaled.

DbItem supports some other "complex" types.

- Orbit/Sv health (in form of a `bitflag` (binary flags))
  - `Health`: health status report used along GPS constellation
  - `GalHealth`: health status report for GAL NAV 
  - `IrnssHealth`: health status report for IRN NAV
  - `GeoHealth`: health status report for SBAS NAV
