# RINEX Observation Data

Observation (OBS) `RINEX` is amongst the two most standard `RINEX` formats.  

Observation files are made of physical measurements:

* `Pseudo Range` estimates (m)
* `Signal Strength` (dBm)
* `Doppler` : doppler shift observation (n.a)
* `Phase` : raw carrier phase measurements (n.a)

that may be avaiable at a certain `epoch` and some may only
be evaluated for specific `sv`.

One example script is provided
```bash
cargo run --example observation
```

### CRINEX - Compressed OBS records

Compressed OBS records are not fully parsed at the moment because
this lib is not able to decompress them.  
You have to manually decompress them prior anything.  
If you pass a compressed RINEX to this lib, only the header
will be parsed and the record is set to _None_.  
Link to CRX2RNX [official decompressoin tool](https://terras.gsi.go.jp/ja/crx2rnx.html)

Some uncompressed V3 OBS files result in format violations,
where _epochs_ are described in a single line longer than 60 characters.   
This parser can deal with that scenario and should parse every sane epoch.

## Observation Record content

The OBS records are sorted by `Epoch` : observation timestamps and `EpochFlags`.   
An `Epoch` comprises all observations per `Sv` and a clock offset (f32) value.   
Observations per _Sv_ are finally sorter per standard RINEX observation codes.

Grab an OBS record from a given file:

```rust
let rinex = rinex::Rinex::from_file("observation.rnx")
    .unwrap();
let record = rinex.record
    .unwrap() // record is optional, is None in case of CRINEX
        .as_obs() // observation record cast
        .unwrap();
```

Retrieve all `L1C` Pseudo Range measurements for `E04` specific vehicule
accross all epochs, by using the standard code:

```rust
let data = rinex.record
    .iter()
    .map(|(_, sv)| {
    
    })
    .flatten()
    .unwrap();
```

Retrieve all Pseudo Range measurements (all matching standard RINEX codes),
for the same vehicule, accross all epochs:



[ ] data rescaling ?   
[ ] decimation   
[ ] carrier compensations ?   
[ ] epoch filters
[ ] only Ok
[ ] power failures etc..etc..

