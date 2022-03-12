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

### V3 and following revisions

&#9888; &#9888; This parser expects single line epochs,   
that is line length larger than 60 caracters,    
which is against V < 3 specifications (only allowed in modern RINEX).

&#9888; &#9888; If V > 2 Observation Records with
multi line epochs do exist,    
this lib will not parse them properly
at the moment.

### CRINEX - Compressed Observation records

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
Observations per _Sv_ are finally sorted per standard RINEX observation codes.

Grab an OBS record from a given file:

```rust
let rinex = rinex::Rinex::from_file("observation.rnx")
    .unwrap();
let record = rinex.record
    .unwrap() // record is optional, is None in case of CRINEX
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

TODO
