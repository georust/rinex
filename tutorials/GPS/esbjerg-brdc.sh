#!/bin/sh
# Real time (BRDC) surveying
# Compare this to post processed +3weeks

# Comment out one step to remove it from the synthesized report.
# Change the configuration to modify the navigation setup.
DATA_DIR=test_resources

# Example:
#   GPS <28 : any other is dropped
#  L1/L5 PR only: not using L2, not PPP compatible
FILTER="GPS;<G28;C1C,C5Q"
# Example: skip first hour of that day
TIMEFRAME=">2020-06-25T01:00:00 GPST"
CONF=tutorials/config/survey/cpp_kf.json

# Analysis + ppp solutions (silent)
./target/release/rinex-cli \
    -P $FILTER \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp -c $CONF
