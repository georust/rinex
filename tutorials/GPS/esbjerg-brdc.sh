#!/bin/sh
# BRDC (""real time"") surveying using GPS=L1 pseudo range example
DATA_DIR=test_resources
# Example:
#   GPS >11 : any other is dropped
#    L1 only: not using L2 or L5
FILTER="GPS;>G11;C1C"
# Example: skip last hour of that day
TIMEFRAME="<2020-06-25T23:00:00 UTC"
CONF=tutorials/config/survey/spp_lsq.json # Basic SPP+LSQ

./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp -c $CONF
