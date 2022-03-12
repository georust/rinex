# RINEX Meteo Data

Meteo Observation `RINEX` describes environmental
related measurements.

## Meteo Record content

Grab an record from a given file:

```rust
let rinex = rinex::Rinex::from_file("data/MET/V2/cari0010.07m")
    .unwrap();
let record = rinex.record
    .unwrap() // record is optional
        .as_meteo() // meteo record cast
        .unwrap();
```

## Meteorological sensors information

`rinex.header` contains some sensor informations
specific to these files:

* `rinex.header.sensors` : a list of `Sensor` structure
to describe the hardware responsible for a specific
observation code

## Record manipulation

The observations are sorted by `Epoch` and by Observation Code.

### Observation Codes

It is very important to be familiar with Observation Codes to manipulate
OBS data:

* understand what they represent (see RINEX specifications)
* determine which one(s) you are interested in

Observation Codes are listed in the `rinex.header` structure
as `String` values:

```rust
let obs_codes = &rinex.header.met_codes
	.unwrap(); // met_codes are optionnal,
		// because they're not available
		// to other RINEX types

[ // example
    "PR",
    "HR",
    "TD",
]
```

These codes are then used to retrieve data of interest.   
It is important to understand which type of measurement
these codes represent.
