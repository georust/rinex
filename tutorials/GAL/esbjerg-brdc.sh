#!/bin/sh
# Real time surveying (BRDC)
# Compare this to post processed +3weeks
DATA_DIR=test_resources

# Comment out one step to remove it from the synthesized report.
# Change the configuration to modify the navigation setup.
DATA_DIR=test_resources

# Example: 
#  Gal PRN>09
#  Skip 1st hour
#  E1 or E5 PR
FILTER="Gal;>E09;C1C,C5Q"
# Skip 1st hour (example)
TIMEFRAME=">2020-06-25T01:00:00 UTC"
CONF=tutorials/config/survey/cpp_kf.json

# Analysis + ppp solutions
#   -f: force new report synthesis
#   -q: silent (open on last call)
#   -o: custom name
./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    -f -o "BRDC-GalE1E5" \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp -c $CONF

# cggtts solutions (+open).
# Since we're using strict identical options,
# the report is preserved and new solutions are appended.
# The report is automatically opened.
