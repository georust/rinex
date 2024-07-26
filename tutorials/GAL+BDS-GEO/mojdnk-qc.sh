#!/bin/sh
# Standalone QC example (with BDS-GEO and extra constellation)
DATA_DIR=test_resources
# Example: 
#  Skip 1st hour (example)
#  Gal PRN>09 (example)
#  BDS Any
#    In this example, include BDS (GEO)
#    compare this one to the other script, where we do not
FILTER="Gal,BDS;>E09;C1C,C5Q,C2I,C6I,C7I"
# Skip 1st hour (example)
TIMEFRAME=">2020-06-25T01:00:00 UTC"

./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    -f $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    qc 
