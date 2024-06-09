#!/bin/sh
DATA_DIR=test_resources
CONF=config/survey/spp_lsq.json

./target/release/rinex-cli \
   -f $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
   -f $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
   -P GPS -p -c $CONF | tee logs/mojn-gps+spp.txt
