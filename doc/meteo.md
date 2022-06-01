# RINEX Meteo Data

Meteo Observation `RINEX` describes environmental measurements.

One example script is provided

```bash
cargo run --example meteo 
```

Refer to this example to learn how to browse MET `record, that includes:
* [x] grab high level information
* [x] enumerate epochs contained in record
* [x] retrieve specific data.. 

It is important to be familiar with Observation Codes to manipulate
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

## Meteorological sensors information

`rinex.header` contains some sensor informations
specific to these files:

* `rinex.header.sensors` : a list of `Sensor` structure
to describe the hardware responsible for a specific
observation code
