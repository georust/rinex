# RINEX Observation Data

Observation (OBS) `RINEX` is amongst the two most standard `RINEX` formats.  

Observation data comprises several kinds of data

* `Pseudo Range` estimates (m)
* `Signal Strength` (dBm)
* `Doppler` : doppler shift observation (n.a)
* `Phase` : raw carrier phase measurements (n.a)

## Observation Record analysis

All records are sorted by observation `Epoch`.  

Measurements are exposed using standard Observation Codes, refer
to official RINEX documentation.

### List all Epochs

```rust
let epochs = rinex.record.iter()
    .unique()
    .collect(); 
println!("{:#?}", epochs);
```

`unique()` to filter unique `Epoch` values is not really needed here,
since a sane OBS Rinex will have a single realization per epoch,
for a specific satellite vehicule and a specific measurement.

### Extract `Pseudo Ranges` for all `Sv` across all epochs

todo

### Extract `Pseudo Ranges` for specific `Sv` across all epochs

todo

Restraint to a specific epoch:

todo

### Decimate `Phase` data of a specific `Sv` using epoch /sampling interval

todo
