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

### Analysis and report files

Analysis and reports are generated in HTML, in the `rinex/rinex-cli/product` directory.  
Analysis is named after the primary RINEX file, so it is possible to generate
several products and keep them.

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

Grab the binary for your architecture 
[from the latest release](https://github.com/gwbres/rinex/releases).

Or compile the application manually:

```shell
cargo build --release
./target/release/rinex-cli -h
```

From now on, "rinex-cli" means "target/release/rinex-cli" previously compiled.  

All examples depicted in this documentation suite uses our
[test data](https://github.com/gwbres/rinex/tree/main/test_resources).  
That means you have everything to reproduce the provided examples on your side. 

## Command line interface

File paths have to be absolute.   
Arguments order does not matter to this application: 

```bash
rinex-cli --fp test_resources/NAV/V2/amel010.21g
rinex-cli --sv-epoch --fp /tmp/amel010.21g
```

Some operations may require an argument. In this case we expect a CSV description,
for example, `--retain-sv` to focus on vehicles of interest is one of those:

```bash
rinex-cli --fp rovn0010.21o --retain-sv G01,G02
```

As previously said, [rinex-cli/product](product/) is where we generate
analysis reports.

Analysis are stacked to one another, and order does not matter.  
For example, when providing an Observation RINEX data, 
one plot per physic is to be generated, and here we request
two other analysis to be performed:

```bash
rinex-cli \
    --retain-sv R01,R08,R19,G08,G21,G31 \ # focus
    --sv-epoch \ # Sv per Epoch identification
    --epoch-hist \  # sampling rate histogram analysis
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz 

ls rinex-cli/product/ESBC00DNK_R_20201770000_01D_30S_MO/*
```

## HTML content

The analysis report is generated in HTML.  
In the future, we will allow other formats to be generated
(like JSON). 

When the analysis is concluded, the report
is opened in the default web browser. This is turned off
if the quiet (`-q`) is active.

The HTML content is self sufficient, all Javascript
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
rinex-cli -f KOSG0010.95O --epoch
``` 

As always, Identification operations can be stacked together, to perform several at once.
For example, identify encountered vehicules at the same time:

```bash
rinex-cli -f test_resources/OBS/V2/KOSG0010.95O --epoch --sv
``` 

Basic operations like these only output to "stdout" currently.  
The `--pretty` option is there to make the datasets more readable: 

```bash
rinex-cli -f test_resources/OBS/V2/KOSG0010.95O --epoch --sv --pretty
``` 

## Data analysis

Several analysis can be stacked to the generated report, 
like `--sv-epoch` or sample rate analysis with `--epoch-hist`.   
Refer to their [dedicated page](doc/analysis.md) documentation.

## Record analysis

When analyzing a RINEX, it is probably needed to reduce
the file content and focus on data you're interested in.

We developed several filter operations, from which we
distinguish two categories:

* [Filtering operations](doc/filtering.md) 
* [Resampling operations](doc/resampling.md) 

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

### File Header customization

A header section customization interface is currently under development.  
It is possible to pass custom header fields, one per `--output` flag,
to customize such section of the RINEX to be generated.

The `custom-header` flag accepts either a direct JSON description
of the `Rinex::Header` structure, or a local file containing
such a description.
