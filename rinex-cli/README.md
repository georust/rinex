RINEX-cli 
=========

Command line tool to parse, analyze and manage RINEX files.  

[![crates.io](https://img.shields.io/crates/v/rinex-cli.svg)](https://crates.io/crates/rinex-cli)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 

The first purpose of this tool is to expose the [library](https://github.com/gwbres/rinex/rinex) 
in a high level and easy to use fashion.  
The application will be able to parse all RINEX formats supported by the library, refer to the front page to understand which RINEX format is currently supported.

Some GNSS data processing algorithms are implemented,
like 
- signal recombination
- processing (DCBs, MP, ...)

Refer to [the dedicated page](doc/processing.md)

Some `teqc` operations are supported, see the 
[following paragraph](https://github.com/gwbres/rinex/blob/main/rinex-cli/README.md#teqc-operations)

## RINEX files

Several RINEX files exist. The tool support most RINEX formats, some exotic formats
are still under development, refer to the 
[main table](https://github.com/gwbres/rinex/blob/main/README.md#supported-rinex-types)

### File naming conventions

File names are disregarded by these tools, you can analyze
& parse files that do not follow naming conventions.

When producing data, this tool will eventually help the user to generate RINEX that follows
naming conventions, but that is currently under development.

### Compressed data

CRINEX (V1 and V3) are natively supported.  
This tool supports gzip compressed files, as long as their name is terminated by `.gz`.

## Data visualization & analysis

Most data analysis, like DCBs or RINEX record analysis produce plots.  
Plots currently come in the form of PNG files.

Efficient plotting is tied to efficient data filtering and resampling,
because RINEX files contain a lot of data.

For example, this is the "ssi.png" generate during an Observation analysis. 

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_glo_ssi.png">

Data is most of the time plotted against time.   
Time axis represents UTC epochs, normalized to 1st epoch (starting a "0"),
and expressed in seconds. 

## `teqc` operations

`teqc` is a well known application to process RINEX.   
Unlike teqc, this application is not capable of processing Binary RINEX ("BINEX") and 
proprietary formats in general.

Some teqc operations are supported:

- [merge](doc/merge.md) several RINEX together into a single file.
- [split](doc/split.md) given RINEX into two
- [resampling](doc/sampling.md) to reduce data quantity
- [quality check](doc/qc.md): file quality check, mainly Observations

## Getting started

Install dependencies:

```shell
apt-get install libfontconfig1-dev
```

Always compile Rust code with the `--release` flag for optimized implementation.

```shell
cargo build --release
./target/release/rinex-cli -h
```

From now on, "rinex-cli" means "target/release/rinex-cli" previously compiled.  
This tool expects a primary RINEX with `--fp`, and actually is the only mandatory argument.  
Also, all provided examples are run against our 
[test data](https://github.com/gwbres/rinex/tree/main/test_resources)
for the user to reproduce easily. We sometimes omit the absolute path for simplicity.

```bash
rinex-cli -fp amel010.21g
```

File paths have to be absolute. 
Arguments order does not matter to this application: 

```bash
rinex-cli --sv-epoch --fp /tmp/amel010.21g
```

Some arguments may require user options, in this case we expect a CSV description.   
For example, `--retain-sv` (focus filter) is one of those:

```bash
rinex-cli --fp rovn0010.21o --retain-sv G01,G02
```

There are three main folders when operating this tool

* [rinex-cli/product](product/) is where we generate files, reports..
* [rinex-cli/config](config/) is where we expect configuration files.
Configuration files will fine tune the tool when performing advanced operations.

By default, the tool will analyzes the (`--fp`) RINEX record content.   
It is a graphical visualization of the file content.   
The visualization (type of plots) depends on the type of RINEX that was provided (`--fp`). 

For example, when providing an Observation file like this, 
one plot per physic is to be generated. 
Stacking more operations will either generate more plots or other reports.

```bash
rinex-cli \
    --retain-sv R01,R08,R19 \ # focus
    -w "2020-06-25 03:00:00 2020-06-25 05:00:00" \ # time focus
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz 

ls rinex-cli/product/ESBC00DNK_R_20201770000_01D_30S_MO/*
```

Basic RINEX identification is triggered by requesting such an operation,
like epoch enumeration (`--epoch`) or Sv identification (`--sv`).
Basic operations like these ones simply generate a terminal output,
refer to [this paragraph](https://github.com/gwbres/rinex/tree/main/rinex-cli/README.md#terminal-output)
for more detail.

### Terminal output 

This tool uses JSON format to expose data by default. 
This makes it easy to import into other tools.  

For instance, `eval()` in Python can evaluate complex structures directly.

The `--pretty` argument improves the rendering of JSON data and makes it more readable.

```bash
rinex-cli -f /tmp/amel010.21g --epoch
rinex-cli --pretty -f /tmp/amel010.21g --epoch
``` 

## Data identification

Basic Identification consists in extracting high level information to understand which 
data is contained in a given RINEX.
Examples of such information would be `Epoch` or `Sv` enumerations.

For example:

```bash
rinex-cli -f KOSG0010.95O --epoch
rinex-cli -f KOSG0010.95O --epoch --pretty
``` 

As always, Identification operations can be stacked together, to perform several at once.
For example, identify encountered vehicules at the same time:

```bash
rinex-cli -f test_resources/OBS/V2/KOSG0010.95O --epoch --sv --pretty
``` 

## Data analysis

Several analysis can be performed, like `--sv-epoch` or sample
rate analysis with `--epoch-hist`. When performing data analysis,
a graphical output (plot) is prefered. 

Refer to the [analysis mode](doc/analysis.md) documentation.

## Record analysis

When data identification or basis analysis are not requested,
the tool is set to record analysis. This mode is the default mode.

[Filtering](doc/filtering.md) or [Resampling](doc/resampling.md) 
operations can be stacked to Record analysis,
to focus on data of interest.

Move on to the [record analysis mode](doc/record.md) for thorough
examples of RINEX record manipulations.

## File generation

Like input files, output files do not have to follow RINEX naming conventions.  
Eventually, this tool might have a file creation helper, to help the user
follow naming conventions, but it is currenly under development.

File generation applies to all the following operations

* [filtering](doc/filtering.md): create a new RINEX from the stripped RINEX content
* [merge](doc/merge.md): merge two files into a single RINEX
* [split](doc/split.md): split a file into two.

For example, let's extract G01 from this file

```bash
rinex-cli -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --retain-sv \
        --output g01.txt
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

### Header section customization

A header section customization interface is currently under development.  
It is possible to pass custom header fields, one per `--output` flag,
to customize such section of the RINEX to be generated.

The `custom-header` flag accepts either a direct JSON description
of the `Rinex::Header` structure, or a local file containing
such a description.
