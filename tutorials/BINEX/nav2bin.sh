#!/bin/sh
# NAV RINEX input to BINARY
DATA_DIR=test_resources

# auto generated filename (standard.bin)
# gzip: force gzip output
./target/release/rnx2bin \
    --gzip \
    $DATA_DIR/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx
