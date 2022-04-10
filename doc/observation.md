# RINEX Observation Data

Observation (OBS) `RINEX` is amongst the two standard `RINEX` formats.  

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

Refer to this example to learn how to browse OBS `records`, that includes:
* [x] grab high level information
* [x] enumerate epochs contained in record
* [x] enumerate epochs for specific `Sv` contained in record
* [x] retrieve specific data, for specific `Sv` and specific `epochs`..

### CRINEX - Compressed Observation records

Compressed OBS records can now be parsed directly.   
You can pass a CRINEX (V1 or V3) directly to this parser and
it will directly evaluate its content.   
This use case is the less efficient of any other format or combination,
because the CRINEX must be decompressed internally prior parsing.  

### Special note for modern (V3+) Observation Data files

&#9888; &#9888; This parser only works with modern OBS files
that either were decompressed with `CRX2RNX` (official tool) 
or my 
[Hatanaka command line tool](https://github.com/gwbres/hatanaka).   
Indeed, `CRX2RNX` does not respect RINEX definitions, because
its producess unwrapped (>80) lines for V3 decompression. 

Next release will support both cases:
* make the standard definition properly parsed (wrapped lines)
* and maintain current behavior, so non matching but very common
files produced by `CRX2NRNX` are still parsed correctly
