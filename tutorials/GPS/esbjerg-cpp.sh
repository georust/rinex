#!/bin/sh
# Post processed (+3week) surveying using GPS=L1/L5 pseudo range example
DATA_DIR=test_resources
# Example:
#   GPS <28 : any other is dropped
#  L1/L5only: not using L2
FILTER="GPS;<G28;C1C,C5Q"
# Example: skip first hour of that day
TIMEFRAME=">2020-06-25T01:00:00 GPST"
CONF=tutorials/config/survey/cpp_kf.json # PR;filter=kalman;x17

./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp -c $CONF
