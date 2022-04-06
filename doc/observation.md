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

In the next release, I will support both cases:
* make the standard definition properly parsed (wrapped lines)
* and maintain current behavior, so non matching but very common
files produced by `CRX2NRNX` are still parsed correctly

## Observation Record content

The OBS records are sorted by `Epoch` : observation timestamps and `EpochFlags`.   
An `Epoch` comprises all observations per `Sv` and a clock offset (f32) value.   
Observations per _Sv_ are finally sorted per standard RINEX observation codes.

Grab an OBS record from a given file:

```rust
let rinex = rinex::Rinex::from_file("observation.rnx")
    .unwrap();
let record = rinex.record
    .as_obs() // observation record cast
    .unwrap();
```

## Record manipulations

The `record` is first indexed by `epoch` then by `Sv` and finally
by Observation codes.   

### Observation Codes

It is very to be familiar with Observation Codes to manipulate
OBS data:

* understand what they represent (see RINEX specifications)
* determine which one(s) you are interested in

Observation Codes are contained in the `rinex.header` structure,
and are sorted by `Constellation` systems.   
For example, let's determine which observation codes
are available to `Glonass` system in this record:

```rust
let obs_codes = &rinex.header.obs_codes
	.unwrap() // obs_codes are optionnal,
		// because they're not available
		// to other RINEX types
	[&Constellation::Glonass]; // obs_codes are sorted
		// by constellation system

[ // example
    "C1C",
    "L1C",
    "D1C",
    "S1C",
    "C2P",
    "L2P",
    "D2P",
    "S2P",
    "C2C",
    "L2C",
    "D2C",
    "S2C",
]
```

These codes are then used to retrieve data of interest.   
It is important to understand which type of measurement
these codes represent.

### Basic but direct `HashMap` indexing

`hashmaps` allow direct indexing. First retrieve a given `epoch`:

```rust
let epoch_to_match = epoch::Epoch::new(
	epoch::str2date("22 03 04 00 00 00").unwrap() // 2022 03 04 midnight
	epoch::EpochFlag::Ok); // epoch flag
let matched_epoch = &record[&epoch_to_match];

{ // example: 2022/03/04 00:00:00
    Sv { // got 1 SV
        prn: 17, // R17
        constellation: Glonass,
    }: {
        "D2P": 3143.093,
        "D1C": 4041.114,
		[..] // got some measurements
    },
    Sv { // got another Sv
        prn: 2, // R02
        constellation: Glonass,
    }: {
        "D1C": 2936.229,
        "D2P": 2283.736,
		[..]
    },
[..]
```

Within that epoch, retrieve all data for `R24` vehicule:
```rust
let sv_to_match = record::Sv::new(
	constellation::Constellation::Glonass, 
	24); // R24
let matched_sv = &matched_epoch[&sv_to_match];
```

For that vehicule, retrieve "C1C" : raw carrier phase measurement:

```rust
let data = &matched_sv["C1C"];
```

## Advanced manipulation using iterators & filters 

Refer to doc/navigation section as they are fairly similar,   
until this section gets created.

Grab an OBS record:

List all observation codes to be expected:
You can see those are modern V >= 3 observation codes.

List all epochs encountered in this file

(A) Retrieve all Carrier L1 phase and Carrier L1 signal strength,
from all vehicules:
To do so, we use retrieve all L1C and S1C measured data. 

(B) Same thing but we also grab L2 so we get a group of 4 vector,
Carrier phase and signal strength for both L1 and L2 carriers.

Grab all L1 carrier signal strength from E04 sattelite vehicule:
To do so, we use the (A) filter but add an `Sv` filter on top of it.

Grab all data for E04 vehicule that corresponds to
a Doppler Observation code. To do so,
we use one of the `is_doppler_obs_code!()` maccro
and test each observation code against it. We
only retain the payload on test match.

Decimate L1 phase carrier by using a minimum epoch interval.
