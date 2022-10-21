CRX2RNX 
=======

[![crates.io](https://img.shields.io/crates/v/crx2rnx.svg)](https://crates.io/crates/crx2rnx)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/hatanaka/rinex/main/LICENSE-MIT) 

`CRX2RNX` is a command line tool to decompress Compact RINEX into `RINEX` data.  
It is an alternative to the existing official tool.

## Compression algorithm

*Yuki Hatanaka* created a simple yet efficient method to compress
RINEX files, it's called CRINEX,   
latest revision is `CRINEX3` and is specified 
[here](https://www.gsi.go.jp/ENGLISH/Bulletin55.html).

This tool supports both CRINEX1 and CRINEX3 (latest).

This tool supports a maximal compression order of 6, 
which is more than other existing tools: this means
you can safely pass CRINEX that were compressed with other 
existing tools.

## Getting started 

Build with the `--release` flag for best performances.

Specify one file to decompress with `--fp`:

```bash
crx2rnx --fp ../test_resources/CRNX/V1/wsra0010.21d
```

This generates `test_resources/CRNX/V1/wsra0010.21o`, 
to follow `RINEX` naming conventions.   

You can specify the output path yourself:
```bash
crx2rnx --fp ../test_resoures/CRNX/V1/wsra0010.21d \
    -o /tmp/output.rnx # does not have to follow naming conventions
```

```bash
crx2rnx -f ../test_resoures/CRNX/V3/ACOR00ESP_R_20213550000_01D_30S_MO.crx \
    -o /tmp/output.rnx # standard V3 extension
```
