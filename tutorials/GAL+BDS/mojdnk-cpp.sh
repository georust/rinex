#!/bin/sh
# Post processed (+3 week) surveying of the MOJN (DNK) lab station
DATA_DIR=test_resources
# Example: 
#  Skip 1st hour (example)
#  Gal PRN>09 (example)
#  BDS PRN>05 <58 (example: no GEO)
#    In this example, we do not consider BDS(Geo) satellites
#    compare this one to the other script, where we do
FILTER="Gal,BDS;>E09;>C05;<C58;C1C,C5Q,C2I,C6I,C7I"
# Skip 1st hour (example)
TIMEFRAME=">2020-06-25T01:00:00 UTC"
CONF=tutorials/config/survey/cpp_kf.json # pseudo-range(e1/e5); filter:kalman

./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    --fp $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp -c $CONF
