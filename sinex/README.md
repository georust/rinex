SINEX
=====

[![crates.io](https://img.shields.io/crates/v/sinex.svg)](https://crates.io/crates/sinex)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT)

## Parser

`SINEX` special files parsing

[data/](data/) contains several example `SINEX files, mainly for testing purposes

Known behavior:

* this parser does not care about file naming conventions
* the parser expects proper Header and SINEX structures
* the parser only cares about the Header position
* the parser does not care about the Description / Solution order
* the parse does not care about the Description fields order, as specified by standards
