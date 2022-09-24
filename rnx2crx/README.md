RNX2CRX
=======

[![crates.io](https://img.shields.io/crates/v/rnx2crx.svg)](https://crates.io/crates/rnx2crx)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/hatanaka/rinex/main/LICENSE-MIT) 

`RNX2CRX` is a command line tool to compress `RINEX` data into
Compact RINEX. It is an alternative to the existing official tool.

This tool only support CRINEX1 encoding at the moment. 

## Supported revisions

* [x] CRINEX1 
* [ ] CRINEX3 

CRINEX2 was never released

## CRINEX

RINEX Compression is an algorithm designed for **Observation Data** uniquely.

## Note on compression order

This tool supports a compression order of 6.    
It currently limits to 3 to always produce identical data to the existing
official tools that are limited to this value.

According to Y. Hatanaka's publication, 
optimum compression performances are obtained for a compression order of 4.
