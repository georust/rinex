RINEX Cli 
=========

[![Rust](https://github.com/gwbres/rinex-cli/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex-cli/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/rinex-cli.svg)](https://crates.io/crates/rinex-cli)
[![crates.io](https://img.shields.io/crates/d/rinex-cli.svg)](https://crates.io/crates/rinex-cli)   
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 

Command line tool to handle, manage and analyze RINEX files

## RINEX

Why this tool ?

`RINEX` files are very common, they are used in 
* GNSS applications :artificial_satellite:
* Timing applications :clock1:
* Astronomy :satellite:
* Space applications :rocket: 
* Navigation :earth_americas:

`RINEX` files are complex, several kinds exist and they differ a lot from one another.  
This tool is powerful enough to manage almost all revisions and most common RINEX files,
without compromising ease of use.

## File naming conventions

File names are disregarded by these tools, you can analyze
& parse files that do not follow naming conventions.

## Compressed files

It is possible to directly analyze Observation Data that previously was
compressed using the official `RNX2CRX` tool.

This tool does not support extra compression (like .gz for instance).
It is up to the user to decompress these files prior analysis.

## Getting started

This application is managed by `cargo`, therefore `cargo run` is one way to run it

```bash
cargo run -- --help
```

Command line arguments order does not matter.  
(Input) `filepath` is the only mandatory argument, other flags are optionnal.
`Help` menu tells you which argument has a shortenned version,
here is an example on how to use a shortenned argument:

```bash
cargo run --filepath /tmp/amel010.21g
cargo run -f /tmp/amel010.21g
```

Some arguments, like `filepath` or `obscodes` can take an array of values.
In this case, we use comma separated enumeration like this:

```bash
cargo run -f /tmp/amel010.21g,/mnt/CBW100NLD_R_20210010000_01D_MN.rnx
```

## Output format

This tool display everything in the terminal (`stdout`).
One should pipe the output data to a file in order to store it.

This tool uses JSON format to expose data, which makes it easy
to import into external tool for further calculations and processing,
like python scripts for instance.

At any moment, add the `--pretty` option to make this more readable if desired.
Output is still valid JSON.

## Epoch identification 

`--epoch` or `-e`
is used to export the identified epochs (sampling timestamps)
in the given RINEX records.
When we extract data, we always associate it to a sampling timestamp.
Therefore, this flag should only be used if the user studies
epoch events or sampling events specifically.

Example :

```bash
cargo run -- -f /tmp/data.obs --epoch --pretty
cargo run -- -f /tmp/data.obs -e > /tmp/epochs.json
```

## OBS / DATA code identification

`--obscodes` or `-o`
is used to identify which data codes (not necesarily OBS..) 
are present in the given records.
This macro is very useful because it lets the user
understand which data (physics) is present and we can build
efficient data filter from that information

```bash
cargo run -- -f /tmp/data.obs --obscodes --pretty
cargo run -- -f /tmp/data.obs -o > /tmp/data-codes.json
```

This flag name can be misleading as it is possible to use this
flag to identify NAV data field also !

## Record resampling

Record resampling is work in progress !

## Epoch filter

Some RINEX files like Observation Data 
associate an epoch flag to each epoch.  
A non `Ok` epoch flag describes a special event
or external pertubations that happened at that sampling
date. We provide the following arguments to
easily discard unusual events or focus on them to
figure things out:

* --epoch-ok : will retain only valid epochs (EpochFlag::Ok).
This can be a quick way to reduce the amount of data

* --epoch-nok : will retain only non valid epoch (!EpochFlag::Ok). 

Example :

```bash
cargo run -- -f /tmp/data.obs -c C1C,C2C # huge set
cargo run -- -f /tmp/data.obs --epoch-ok -c C1C,C2C # reduce set 
cargo run -- -f /tmp/data.obs --epoch-nok -c C1C,C2C # focus on weird events 
```

## Satellite vehicule filter

`--sv` is one way to focus on specific Satellite Vehicule or constellation of interest.
Use comma separated description!

Example:

```shell
cargo run -- --filepath /tmp/data.obs --epoch-ok \
    --sv G01,G2,E06,E24,R24
```

Will only retain (all) data that has a EpochFlag::Ok 
from GPS 1+2, GAL 6+24 and GLO 24 vehicules.

## Constellation filter

Constellation filter is not feasible at the moment.
User can only use `sv` filter to do that at the moment.

## LLI : RX condition filter

Observation data might have LLI flags attached to them.  
It is possible to filter data that have a matching LLI flag,
for instance 0x01 means Loss of Lock at given epoch:

```shell
cargo run -- -f /tmp/data.obs --lli 1 --sv R01 > output.txt
```

## SSI: signal "quality" filter

Observation data might have an SSI indication attached to them.
It is possible to filter data according to this value and
retain only data with a certain "quality" attached to them.

For example, with this value, we only retain data with SSI >= 5
that means at least 30 dB SNR 

```shell
cargo run -- -f /tmp/data.obs --ssi 1 --sv R01 > output.txt
```

## Data filter

We use the -c or --code argument to filter data out
and only retain data of interest.

* Observation / meteo data: we use the OBS code directly
* Navigation data: we use the official identification keyword,
refer to the known
[NAV database](https://github.com/gwbres/rinex/blob/main/navigation.json)


Example:

```bash
cargo run -f CBW100NLD_R_20210010000_01D_MN.rnx -c L1C,S1P 
```

## Cummulated filters

Because all arguments can be cummulated, one can 
create efficient data filter and focus on data of interest: 

```bash
cargo run -f CBW100NLD_R_20210010000_01D_MN.rnx \
    --lli 0 # "OK" \
        --ssi 5 # not bad \
            -c C1C,C2C,C1X # PR measurements only :) \
                --sv G01,G2,G24,G25 # GPS focus !
```

## `teqc` operations

This tool supports special operations that only
`teqc` supports at the moment. Therefore
it can be an efficient alternative to this program.

All of the special operations actually create an output file.

## `Merge` special operation

It is possible to perform merging operations with `-m` or `--merge`, in `teqc` similar fashion.

When merging, if analysis are to be performed, they will be performed on the resulting record.

For example:

```bash
cargo run -f file1.rnx,/tmp/file2.rnx
```

## `Split` special operation

It is possible to split given RINEX files into two.

`Split` has two behaviors

* (1) if given RINEX is a `merged` RINEX (either by `teqc` or this tool),
we split the record at the epoch (timestamps) where records were previously merged

* (2) otherwise, the user is expected to describe a timestamp (`epoch`) 
at which we will split the record.

### Splitting a previously merged record

```bash
# Merge two RINEX toghether
cargo run -f /tmp/file1.rnx,/tmp/file2.rnx -m --output /tmp/merged.rnx
# Split resulting RINEX
cargon run -f /tmp/merged.rnx --split --output /tmp/split1.rnx,/tmp/split2.rnx 
```

When splitting a merged RINEX, the header section is simply
copied into both results.

### Splitting record

If User provides an `epoch`, the tool will try to locate the
given timestamp and perform `split()` at this date & time.

Two description format are supported, for the user to describe a sampling
timestamp:

* "YYYY-MM-DD HH:MM:SS" : Datetime description and EpochFlag::Ok is assumed
* "YYYY-MM-DD HH:MM:SS X" : Datetime description where X describes the EpochFlag integer value.
Refer to RINEX standards for supported Epoch flag values 

The tool identifies matching timestamp by comparing the datetime field AND
the flag field. They both must match.

Example :

```bash
# Split a previously merged record
cargo run -f /tmp/merged.rnx --split \
    --output /tmp/file1.rnx,/tmp/file2.rnx

# Split a record at specified timestamp,
# don't forget the \" encapsulation \" ;)
cargo run -f /tmp/data.rnx --split "2022-06-03 16:00:00" \
    --output /tmp/file1.rnx,/tmp/file2.rnx

# Split a record at specified timestamp with precise Power Failure event
# don't forget the \" encapsulation \" ;)
cargo run -f /tmp/data.rnx --split "2022-06-03 16:00:00 1" \
    --output /tmp/file1.rnx,/tmp/file2.rnx
```
