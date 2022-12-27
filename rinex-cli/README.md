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

For each session, you get a `rinex/rinex-cli/product/$PRIMARY` folder, where $PRIMARY
is the name of the `--fp`  primary RINEX file that was given to the command line.  
  
This allows generating and saving multiple reports in successive tool invokations.

## Getting started

Grab the binary for your architecture 
[from the latest release](https://github.com/gwbres/rinex/releases).

Or compile the application manually:

```shell
cargo build --release
./target/release/rinex-cli -h
```

When we say "rinex-cli", we now imply "target/release/rinex-cli" - previously installed.  

All examples described here are based off our 
[test data](https://github.com/gwbres/rinex/tree/main/test_resources).  
This means you have everything to reproduce the provided command lines on your side.   

## Command line interface

File paths have to be absolute. 
Arguments order does not matter to this application: 

```bash
rinex-cli --fp test_resources/NAV/V2/amel010.21g
rinex-cli --sv-epoch --fp /tmp/amel010.21g
```

`--fp` is the only mandatory flag and is how the "primary" RINEX file is given.  

In advanced modes, we'll see that other (secondary) files can be passed, to unlock
advanced operations. 

Use the `RUST_LOG` environment variable to take advantage of the logger.  
Set the sensitivy as desired, "trace" being the most sensitive,
"info" is the standard value:

```bash
RUST_LOG=trace rinex-cli --fp test_resources/NAV/V2/amel010.21g

export RUST_LOG=info
rinex-cli --fp test_resources/NAV/V2/amel010.21g
```

## HTML and analysis report

When an operation is requested, it gets added to the general report
being generated. Most analysis are graphical. The Quality Check (`--qc`) summary report
is a more verbose approach, where information you may get from graphs analysis
or graphs manipulation are exposed or depicted in another form.  
  
There also are information you only get with the Quality Check summary.   
Refer to its [dedicated section](doc/qc.md).

The report is written in HTML and is supposed to be opened in a web browser.    
In the future, this tool may support other formats.  
  
When the analysis is concluded, the application will try to open the report
that has just been generated, with the "default" web browser.   
Depending on the OS context, this may not be feasible.  
This "automated" feature is turned off when the quiet mode is activated with `-q`.

When graphs are to be included in the report, its size grows significantly.    
In order to share report easily, a `--tiny-html` option is there to reduce
the report size. The reduction factor is approximately 8.  

When this option is used, the plotting (javascript) library is not included in the HTML,  
it will be retrieved from the web when the user first opens such a graph.  
Basically, the report is slower to open but smaller in size.  

Modern web browsers are cache performant, there is litterally no performance degradation
when using this option which might aswell become the standard behavior in the future.
We recommend to use it, especially if you generate a lot of reports, store them and
are interested in some of the graphs we developed.

## Basic operations

TODO

To proceed towards advanced operations, start with the [record analysis](#record-analysis) 
paragraph down below.  

## Filter designer

RINEX files are huge and the information you're interested might not be directly visible.    
This tool also supports a lot of operations. A filter design interface is now introduced to address
that problem.  
  
It is important to master the _filter designer_ to operate this tool efficiently.  
  
A filter is specified with a verbose description, prefixed with `-F`. 

Currently we define them as "preprocessing" filter, they apply prior further analysis
and their mostly used to reduce the whole dataset to an interesting smaller / focused dataset.  
  
One can stack as many filters as desired, they will apply in the specified order.

Three types of preprocessing filter are known to this day

* `mask` filters
* `smoothing` filters
* `interpolation` filters

A filter description has always the form 'type:description'.   
The description is "type" dependent and is described down below.  

### Mask filter

Mask filter are used to zoom in on a data subset or drop non interesting data subsets.    
They support a broad range of targets. Their description always starts with `-F 'mask:`.  
  
Here are a few examples of masking operations:

1. Apply a unique filter, to only keep Sv we're interested in:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -F 'mask:sv:G08,G09,G10'
```

### Supported filter operands

* `gt`:  Greater than (>)
* `geq`: Greater than included (>=)
* `lt`:  Lower than (<)
* `leq`: Lower than included (<=)
* `eq`: Equality condition (=)
* `neq`: Non Equality (!=)

When the operand is omited, "=" is implied, for example these  
are two equivalent filters

```bash
`-F mask:gnss:gps,glo`
`-F mask:eq:gnss:GPS, GLO` 
```

Some operands do not make sense depending on the targeted items.  
That is true for example for the `gnss` categories with `lt` or `gt` operands
for example.  
  
These are examples of invalid filter descriptions

```bash
`-F mask:gt:gps,glo`
`-F mask:geq:GNSS:GPS, GLO` 
```

The '>' and '>=' operands do not apply to GNSS constellations.  
  
One exception exist for the `Sv` target.   
For example, with the following filter, we retain PRN above 08 for the GPS and 15 for Glonass

```bash
`-F mask:gt:sv:G08`
```

### Combining filters

Any amount of filters is supported.  
For example, retain PRN above 08 for GPS and below 15 (included) for Glonass:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -F mask:gt:sv:G08 mask:leq:R15
```

### Filter summary

* `-F 'description'` is how you describe a filter. Use inverted commas  
if $description contains whitespaces  
  
* `-F mask:sv:G08,G09` describes a mask filter targetting vehicles G08, G09.   
The filter operand is omited: "equality" is implied: we'll retain all data from these vehicles.
  
* `-F mask:neq:sv:G08` operand is specified, we'll retain all data from all vehicles but G08.

* `-F mask:gnss:GPS, GAL' `sv` or `gnss` are targeted items.  
See down below for the complete list of known targets.  
When several targets are targeted, the payload is a CSV string and whitespaces are allowed.

* `-F smooth:hatch`  the first keyword describes the filter type.  
  
* `-F mask:gt:gnss:GPS` some operands  

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


## `teqc` operations

`teqc` is a well known application to process RINEX.   
Unlike teqc, this application is not capable of processing Binary RINEX ("BINEX") and 
proprietary formats in general.

Some teqc operations are supported:

- [merge](doc/merge.md) several RINEX together into a single file.
- [split](doc/split.md) given RINEX into two
- [resampling](doc/sampling.md) to reduce data quantity
- [quality check](doc/qc.md): file quality check, mainly Observations
