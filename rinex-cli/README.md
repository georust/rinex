RINEX-cli 
=========

[![crates.io](https://img.shields.io/crates/v/rinex-cli.svg)](https://crates.io/crates/rinex-cli)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 


`rinex-cli` is a command line application to post process RINEX data.  
It can be used to reshape your RINEX files, or perform basic geodesic calculations,
like broadcast ephemeris analysis and differential code biase estimation.

It can also generate an html report that is similar to "teqc" QC mode.

## File loading interface

`rinex-cli` accepts a single post processing RINEX context
to be loaded. Such a context can comprise any kind of RINEX files and
also accept SP3 files.

Feasible operations are highly dependent on the provided context.
Refer to [this page](doc/file-combination.md) to understand
what data context you should provide for the analysis you want to perform.

You have two options to define your work context:

- Use `-f` (or `--fp`) to load as many individual files as you want.
Can either be valid RINEX or SP3 files.
- Use `-d` to load an entire folder recursively
- Both flags can be used together

`rinex-cli` currently only allows a single context to be defined.  
We don't have an interface to define a dual context, for example
a Reference station and a Rover station as needed in true RTK.  
Thefore, it can only resolve the position
of the receiver using standard trilateration method.

## NB: Observation Data in your Context

The command line lets you load as many Broadcast Navigation data as you want,
which makes covering the time frame of your Observation Data very easy.
But special care must be taken when defining your context:

- You should only load coherent Observation Data.
The interface lets you load as many Observation files as you want,
but they should all be sampled by the same station. 
One benefit is you can load and analyze several days at once.

- If you're not defining the receiver/station location yourself,
we search for it in all provided files individually. 
If it's defined in your Observation Data Header, this one is to be prefered.
But sometimes it is not the case. In case you loaded Broadcast Navigation from
different stations, you may wind up using the predefined apriori position,
and all position solver results will not be expressed against the correct location.  
Pay extra attention to the _apriori_ position being used:

- Just like Broadcast Navigation, we allow loading as many SP3 files as you need.  
For very advanced usage, you should only stack coherent SP3 files that used
the same modeling in their making

## File loading example

`rinex-cli` works totally fine with partial contexts, especially loading a single file.
For example, load one Observation File with :

```bash
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz -i
```

Note that :

- File names are disregarded by this tool, you can parse and analyze files
that do not follow naming conventions
- Gzip compressed files must be terminated by .gz
- Seamless CRINEX V1 and V3 decompression is built in

`-f` lets you load any kind of RINEX individually

Example : run basic identification on a single MIXED NAV file :

```bash
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz -i
```

Example : load a coherent OBS + NAV context :

```bash
./target/release/rinex-cli -i \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz
```

Example : let's say you have observations realized at station ASC2300USA,
and want to provide overlapping Navigation context quickly :

```bash
ls -l /foo/2023/256/*

/foo/2023/256/NAV/MIXED/AJAC00FRA_R_20232560000_01D_MN.rnx.gz
/foo/2023/256/NAV/GALILEO/AMC400USA_R_20232560000_01D_EN.rnx.gz
/foo/2023/256/NAV/GALILEO/ANK200TUR_S_20232560000_01D_EN.rnx
/foo/2023/256/NAV/GALILEO/ANK200TUR_S_20232560000_01D_EN.rnx.gz
/foo/2023/256/NAV/GALILEO/MAYG00MYT_R_20232560000_01D_EN.rnx.gz
/foo/2023/256/SP3/ESA0OPSULT_20232551200_02D_15M_ORB.SP3.gz
/foo/2023/256/SP3/ESA0OPSULT_20232560600_02D_15M_ORB.SP3.gz

./target/release/rinex-cli \
    -f /tmp/ASC2300USA_R_20232560000_01D_15S_M0.crx.gz \
    -d /foo/2023/256
```

`-d` is recursive and limited to a maximal depth of 5.

## Reports and Workspace 

Reports are generated in different formats: 
- txt
- stdout 
- GPX tracks
- KML tracks

Plots and QC report are generated in HTML format.

All data produced is saved in the workspace, in a folder that bears the name of this current run.  
The run is named after the filename encountered, with preference for Observation then Navigation files name.

The workspace is set to `$GIT/rinex-cli/workspace` by default, but that can be customized with `-w`.

## `teqc` operations

`teqc` is a well known application to process RINEX files.   
Unlike teqc, this application is not capable of processing Binary RINEX ("BINEX") and 
proprietary formats in general.

Some teqc operations are supported:

- [merge](doc/merge.md): Merge two RINEX files together
- [split](doc/split.md): Split a RINEX file into two
- [resampling](doc/preprocessing.md): Resampling operations can be performed 
if you know how to operate the preprocessing toolkit
- [quality check](doc/qc.md): RINEX data quality analysis (mainly statistics and only on OBS RINEX at the moment)
- other advanced operations are documented in the [processing](doc/processing.md) suite

## Positioning (RTK)

`rinex-cli` integrates a position solver that will resolve the radio receiver location 
the best it can, by post processing the provided RINEX context. 
This mode in requested with `-r` or `--rtk` and is turned off by default.  

To learn how to operate the solver, refer to [the dedicated page](doc/rtk.md).

## Getting started

Download the latest release for your architecture 
[from the release portal](https://github.com/gwbres/rinex/releases).

Or compile the application manually:

```shell
cargo build --all-features --release
```

The program is located in  "target/release/rinex-cli",
which we might simply refer in some examples as `rinex-cli`.

All examples depicted in this documentation suite uses our
[test data](https://github.com/gwbres/rinex/tree/main/test_resources).  
That means you have everything to reproduce the provided examples on your side. 

## Command line interface

Refer to the help menu which provides extensive explanations:

```bash
./target/release/rinex-cli -h
```

File paths always have to be absolute.   
Arguments order does not matter to this application: 

```bash
rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
rinex-cli --sv-epoch --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz 
```

Use the `RUST_LOG` environment variable to setup the logger.
Set the sensitivy as desired, "trace" being the most sensitive,
"info" the standard value:

```bash
RUST_LOG=trace rinex-cli --fp test_resources/NAV/V2/amel010.21g

export RUST_LOG=info
rinex-cli -f test_resources/NAV/V2/amel010.21g
```

Here's an example of traces you might get on a complex run:

```bash
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
        -P GPS L1C,L2L ">G18" "<=G25" \
            decim:4:L1C ">2020-06-25T08:00:00 UTC" "<=2020-06-25T10:00:00 UTC"
 2023-06-10T10:37:30.951Z INFO  rinex_cli::context > antenna position: WGS84 (3582105.291m 532589.7313m 5232754.8054m)
 2023-06-10T10:37:30.988Z TRACE rinex_cli::preprocessing > applied filter "GPS"
 2023-06-10T10:37:31.007Z TRACE rinex_cli::preprocessing > applied filter "L1C,L2L"
 2023-06-10T10:37:31.013Z TRACE rinex_cli::preprocessing > applied filter ">G18"
 2023-06-10T10:37:31.015Z TRACE rinex_cli::preprocessing > applied filter "<=G25"
 2023-06-10T10:37:31.055Z TRACE rinex_cli::preprocessing > applied filter "decim:4:L1C"
 2023-06-10T10:37:31.056Z TRACE rinex_cli::preprocessing > applied filter ">2020-06-25T08:00:00 UTC"
 2023-06-10T10:37:31.057Z TRACE rinex_cli::preprocessing > applied filter "<=2020-06-25T10:00:00 UTC"
 2023-06-10T10:37:31.057Z INFO  rinex_cli                > record analysis
 2023-06-10T10:37:31.057Z TRACE rinex_cli::plot::record::observation > Carrier cycles observations
 2023-06-10T10:37:31.058Z INFO  rinex_cli                            > graphs rendered in "product/ESBC00DNK_R_20201770000_01D_30S_MO/graphs.html"
```

`antenna position: WGS (x, y, z)` was located in the provided context: some advanced operations are feasible.  
A few preprocessing operations were requested with `-P`, you get a trace
for every operation that did apply (correct command line description).  
`> record analysis` means the analysis is starting at this point.   
The location of the graphs that were rendered (if any) is given.

[rinex-cli/workspace](workspace/) is where all analysis reports
get generated. It is named after the main RINEX file (`-fp`) which
allows preserving sessions for different files.

Analysis are stacked to one another, and order does not matter.  

For example, when providing OBS RINEX data, 
one plot per physic is to be generated. 
In this example, we also stack two more analysis,
both of them are graphical, so two more graphs are rendered.

```bash
rinex-cli \
    --sv-epoch \ # Sv per Epoch identification
    --epoch-hist \  # sampling rate histogram analysis
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz 
```

## Identification mode

`-i` is a quick method to print basic information in the terminal.

```bash
rinex-cli -f OBS/V2/KOSG0010.95O -i
rinex-cli -d /tmp/BASE_DIR -i -p
``` 

`-p` (or `--pretty`) makes stdout and JSON output more readable.


## Data analysis

Several analysis can be stacked to the generated report, 
like `--sv-epoch` or sample rate analysis with `--epoch-hist`.   
Refer to their [dedicated page](doc/analysis.md) documentation.

## File generation

Like input files, output files do not have to follow RINEX naming conventions.  
Eventually, this tool might have a file creation helper, to help the user
follow naming conventions, but it is currenly under development.

File generation applies to all the following operations

* [preprocessing](doc/preprocessing.md): create a new RINEX from the stripped RINEX content
that results from preprocessing algorithms
* [merge](doc/merge.md): merge two files into a single RINEX
* [split](doc/split.md): split a file into two.

For example, create a file that is only made of G08 data originally contained in this file:

```bash
rinex-cli -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P G08 \ # see preprocessing toolkit
    --output g01.txt # does not have to follow naming conventions
```

Header section is simply copied and maintained.

### File generation and compression

It is possible compress data to .gz directly, if the specified `--output` 
is terminated by `.gz`.

It is also possible to convert Observation Data to CRINEX directly
* to CRINX1 if the specified `--output` is terminated by a standard YYd termination
* to CRINX3 if the specified `--output` is termined by `.crx`

In cases where CRINEX conversion is feasible, 
it is possible to stack the `.gz` compression on top of it
* YYd.gz to specify a CRNX1 + gz
* crx.gz to specify a CRNX3 + gz

Likewise, the mirror operations are feasible:

* extract `.gz` compressed data and dump it as readable
* extract a CRINEX and dump it as a readable RINEX

### File Header customization

A header section customization interface is currently under development.  
It is possible to pass custom header fields, one per `--output` flag,
to customize such section of the RINEX to be generated.

The `custom-header` flag accepts either a direct JSON description
of the `Rinex::Header` structure, or a local file containing
such a description.
