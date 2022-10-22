RNX2CRX
=======

[![crates.io](https://img.shields.io/crates/v/rnx2crx.svg)](https://crates.io/crates/rnx2crx)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/hatanaka/rinex/main/LICENSE-MIT) 

`RNX2CRX` is a command line tool to compress `RINEX` data into
Compact RINEX. It is an alternative to the existing official tool.

This tool only support CRINEX1 encoding at the moment. 

This tool encodes with a compression of 3, which is equivalent
to other existing tools. This means you can safely
pass a CRINEX compressed by this tool, to other existing 
CRINEX decompressors.

## Supported revisions

* [ ] CRINEX1: under test 
* [ ] CRINEX3 

## Getting started

Build with the `--release` flag for optimzed performances

Provide Observation data with `--fp`

```bash
rnx2crx --fp test_resources/OBS/V2/zegv0010.21o
```

This will produce `test_resources/OBS/V2/zegv0012.21D`
to follow naming conventions.

Control the output path yourself with `--output`

```bash
rnx2crx --fp test_resources/OBS/V2/zegv0010.21o \ 
      --output /tmp/test.txt
```

You can force the CRINEX revision to use yourself,
otherwise we compress V2 to CRINEX1, and modern
Observations (V3, V4..) to CRINEX3:

* force to CRINEX1 with `--crx1`
* force to CRINEX3 with `--crx3`
