CRX2RNX 
=======

[![crates.io](https://img.shields.io/crates/v/crx2rnx.svg)](https://crates.io/crates/crx2rnx)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/hatanaka/rinex/main/LICENSE-MIT) 

`CRX2RNX` is a command line tool to decompress Compact RINEX into `RINEX` files.  
It is an alternative to the existing official tool.

## Compression algorithm

*Yuki Hatanaka* created a simple yet efficient method to compress
RINEX files, it's called CRINEX,   
latest revision is `CRINEX3` and is specified 
[here](https://www.gsi.go.jp/ENGLISH/Bulletin55.html).

For more information on the actual compression algorithm, 
refer to the [hatanaka module](https://crates.io/crates/rinex)
of the library.

## Supported revisions

* [x] CRINEX1 
* [x] CRINEX3  

CRINEX2 was never released

## CRINEX

RINEX Compression is an algorithm designed for **Observation Data** uniquely.

## Getting started

Specify one file to decompress with `--filepath` or `-f`:

```bash
crx2rnx -f ../test_resoures/CRNX/V1/wsra0010.21d
```

This generates `test_resoures/CRNX/V1/wsra0010.21o`, 
to follow `RINEX` naming conventions.   

To change that behavior and specify the output file yourself, use `--output` or `-o`:

```bash
crx2rnx -f ../test_resoures/CRNX/V1/wsra0010.21d \
    -o /tmp/output.rnx # custom location, does not follow naming conventions
```

```bash
crx2rnx -f ../test_resoures/CRNX/V3/ACOR00ESP_R_20213550000_01D_30S_MO.crx \
    -o /tmp/output.rnx # custom location with standard V3 extension
```

## Note on compression order

This tool supports a compression order of 6.    
It will adapt naturally to a CRNX file that was first generated using the official binary,
which is limited to a compression order of 3.  
It will be able to uncompress data that was more compressed, using `rnx2crx` of this repo.

According to Y. Hatanaka's publication, 
optimum compression performances are obtained for a compression order of 4.
