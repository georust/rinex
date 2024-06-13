#!/bin/sh
# BRDC (""real time"") surveying using GPS=L5 pseudo range example
DATA_DIR=test_resources
# Example:
#   GPS >05 : any other is dropped
#    L5 only: not using L1 nor L2
FILTER="GPS;>G05;C5Q"
CONF=config/survey/spp_lsq.json
# Example: skip last hour of that day
TIMEFRAME="<2020-06-25T23:00:00 UTC"

./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    -f $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp -c $CONF | tee logs/mojn-gps+spp.txt
