#!/bin/sh
# Post processed (+3week) surveying using GPS=L2 pseudo range example
DATA_DIR=test_resources
CONF=config/survey/spp_lsq.json
# Example (whole day):
#   GPS <28 : any other is dropped
#   L2 only : not using L1 nor L5
FILTER="GPS;<G28;C2W"

./target/release/rinex-cli \
    -P $FILTER \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -f $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    -f $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp -c $CONF | tee logs/esbjr-gps+spp.txt
