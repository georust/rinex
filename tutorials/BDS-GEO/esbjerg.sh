#!/bin/sh
DATA_DIR=test_resources
CONF=tutorials/config/survey/cpp_lsq.json

# Example:
#  Preserve GEO only
FILTER="BeiDou;<C06,>C55"

./target/release/rinex-cli \
    -P $FILTER \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/Sta21114.sp3.gz
