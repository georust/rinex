RINEX-cli 
=========

Command line tool to parse, analyze and manage RINEX files.  

[![crates.io](https://img.shields.io/crates/v/rinex-cli.svg)](https://crates.io/crates/rinex-cli)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 

The main purpose of this tool is to expose the [library](https://github.com/gwbres/rinex/rinex) 
capabilities, in a high level and easy to use interface.

## RINEX files

Several RINEX files exist. The tool support most RINEX formats, some exotic formats
are still under development, refer to the 
[main table](https://github.com/gwbres/rinex/blob/main/README.md#supported-rinex-types)

### File naming conventions

File names are disregarded by this tool, you can analyze
& parse files that do not follow naming conventions.

When producing data, this tool will eventually help the user to generate RINEX that follows
naming conventions, but that is currently under development.

### Compressed data

CRINEX (V1 and V3) are natively supported.  
This tool supports gzip compressed files, as long as their name is terminated by `.gz`.

### Analysis and report files

Analysis and reports are generated in HTML, in the `rinex/rinex-cli/product` directory.  
Analysis is named after the primary RINEX file.

Some advanced computations and analysis are possible with this tool,
refer to the dedicated sections.

## `teqc` operations

`teqc` is a well known application to process RINEX.   
Unlike teqc, this application is not capable of processing Binary RINEX ("BINEX") and 
proprietary formats in general.

Some teqc operations are supported:

- [merge](doc/merge.md): Merge two RINEX files together
- [split](doc/split.md): Split a RINEX file into two
- [resampling](doc/preprocessing.md): Resampling operations can be performed 
if you know how to operate the preprocessing toolkit
- [quality check](doc/qc.md): RINEX data quality analysis (mainly statistics and only on OBS RINEX at the moment)
- other advanced operations are documented in the [processing](doc/processing.md) serie

## Getting started

Grab the binary for your architecture 
[from the latest release](https://github.com/gwbres/rinex/releases).

Or compile the application manually:

```shell
cargo build --release
./target/release/rinex-cli -h
```

From now on, "rinex-cli" means "target/release/rinex-cli" previously compiled
(always prefer the `released` binary for optimal performances).

All examples depicted in this documentation suite uses our
[test data](https://github.com/gwbres/rinex/tree/main/test_resources).  
That means you have everything to reproduce the provided examples on your side. 

## Command line interface

File paths have to be absolute.   
Arguments order does not matter to this application: 

```bash
rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
rinex-cli --sv-epoch --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz 
```

Use the `RUST_LOG` environment variable to enjoy the env logger.  
Set the sensitivy as desired, "trace" being the most sensitive,
"info" the standard value:

```bash
RUST_LOG=trace rinex-cli --fp test_resources/NAV/V2/amel010.21g

export RUST_LOG=info
rinex-cli --fp test_resources/NAV/V2/amel010.21g
```

For example, here is the trace you get on a complex run (don't worry about
the command line options yet)

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

The `antenna position: WGS (x, y, z)` info means the provided context has a ground position defined. 
This is a prerequisite to some advanced operations.   
A few preprocessing operations were requested with `-P`, you get a trace
for every operation that did apply (correct command line description).  
`> record analysis` means the analysis is being performed.  
The location of the graphs that were rendered (if any) is given.

[rinex-cli/product](product/) is where all analysis reports
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

## HTML content

The analysis report is generated in HTML.  
In the future, we will allow other formats to be generated
(like JSON). 

When the analysis is concluded, the report
is opened in the default web browser. This is turned off
if the quiet option (`-q`) is active.

The HTML content is self-sufficient, all Javascript
and other dependencies are integrated . 
This results in a large file whose rendering is quick.
To change that behavior, the `--tiny-html` option is there. 
The resulting report gets reduced (about 8 times smaller), but
graphical views (if any) are longer to render in the browser.
This is actually only true if the javascript has not been cached
by the web browser.

## Data identification

Basic Identification consists in extracting high level information to understand which 
data is contained in a given RINEX.
Examples of such information would be `Epoch` or `Sv` enumerations.

For example:

```bash
rinex-cli -f OBS/V2/KOSG0010.95O --epochs
rinex-cli -f test_resources/OBS/V2/KOSG0010.95O --epochs --sv
``` 

The `--pretty` option is there to make the datasets more readable (json format): 

```bash
rinex-cli -f test_resources/OBS/V2/KOSG0010.95O --epochs --sv --pretty
``` 

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
