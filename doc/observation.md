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

Compressed OBS records are not parsed at the moment because
this lib is not able to decompress them. You have to manually
decompress them prior parsing them for this lib to fully parse them.

## Observation Record content

The OBS record is sorted by `Epoch` and by `Sv`.  
The actual observations (physical measurements) are indexed by using their standard RINEX codes,
refer to their official definition.

Grab an OBS record from a given file:

```rust
let rinex = rinex::Rinex::from_file("observation.rnx")
    .unwrap();
let record = rinex.record
    .unwrap() // record is optional, is None in case of CRINEX
        .as_obs() // observation record cast
        .unwrap();
```

Retrieve all Pseudo Range measurements for one `E04` vehicule
accross all epochs, by using the `L1C` standard code:
```rust
let data = rinex.record
    .iter()
    .map(|(_, sv)| {
    
    })
    .flatten()
    .unwrap();
```

[ ] data rescaling ?   
[ ] decimation   
[ ] carrier compensations ?   
[ ] epoch filters
[ ] only Ok
[ ] power failures etc..etc..

