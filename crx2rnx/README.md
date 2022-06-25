CRX2RNX 
=======

[![crates.io](https://img.shields.io/crates/v/crx2rnx.svg)](https://crates.io/crates/crx2rnx)
[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/d/crx2rnx.svg)](https://crates.io/crates/crx2rnx)   
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/hatanaka/rinex/main/LICENSE-MIT) 

`CRX2RNX` is a command line tool to decompress `RINEX` files easily.  

*Yuki Hatanaka* created a simple yet efficient method to compress
RINEX files, it's called CRINEX,   
latest revision is `CRINEX3` and is specified 
[here](https://www.gsi.go.jp/ENGLISH/Bulletin55.html).

For more information on the actual compression algorithm, 
refer to the [hatanaka object](https://crates.io/crates/rinex)
of the library.

## Supported revisions

* [x] CRINEX1 
* [x] CRINEX3  

CRINEX2 was never released

## CRINEX

RINEX Compression is an algorithm designed for **Observation Data**.

## Getting started

Specify one file to decompress with `--filepath` or `-f`:

```bash
hatanaka -f ../test_resoures/CRNX/V1/wsra0010.21d
```

This generates `test_resoures/CRNX/V1/wsra0010.21o`, 
to follow `RINEX` naming conventions.   

To change that behavior and specify the output file, use `--output` or `-o`:

```bash
hatanaka -f ../test_resoures/CRNX/V1/wsra0010.21d \
  -o /tmp/output.rnx # custom location, does not follow naming conventions
```

```bash
hatanaka -f ../test_resoures/CRNX/V3/ACOR00ESP_R_20213550000_01D_30S_MO.crx \
  -o /tmp/output.rnx # custom location with standard V3 extension
```

## :warning: File format restrictions

This tool currently follows the official `CRX2RNX` behavior, which does not follow
`RINEX` specifications. `CRN2RNX` produces unwrapped epochs that occupy only one line.

## Epoch events 

`COMMENTS` are preserved through compression / decompression, as you would expect.   
Just like `CRX2RNX`, epochs with special events (flag > 2) are left untouched.  
Therefore, explanations on these epochs events are preserved.

## Compression algorithm & limitations 

This tool uses an M=8 maximal compression order, which should be fine for all CRINEX ever produced,   
considering they were probably produced by `CRX2RNX` which hardcodes an M=5 limitation.   

Unlike `CRX2RNX`, this tool is not limited to an hardcoded M value, 
you can increase the default value if you think higher   
compression will be encountered in a given file: 
```bash
hatanaka -M 10 \
  --filepath ../test_resoures/CRNX/V3/KUNZ00CZE.cnx # increase maximal compression order
```

According to Y. Hatanaka's publication, 
optimum compression performances are obtained for a 4th order compression,   
therefore with default parameters.
