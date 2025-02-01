#!/bin/sh
# Real time surveying (BRDC)
# Compare this to post processed +3weeks
DATA_DIR=test_resources

# Example: 
#  Gal PRN>09
#  Skip 1st hour
#  E1 or E5 PR
FILTER="Gal;C1C,C5Q"
CONF=tutorials/config/survey/cpp_kf.json

# Analysis + ppp solutions
#   -f: force new report synthesis
#   -q: silent (open on last call)
#   -o: custom name
./target/release/rinex-cli \
    -P $FILTER \
    -f -q -o "BRDC-GalE1E5" \
    --fp $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp -c $CONF

# cggtts solutions (+open).
# Since we're using strict identical options,
# the report is preserved and new solutions are appended.
# The report is automatically opened.
./target/release/rinex-cli \
    -P $FILTER \
    -o "BRDC-GalE1E5" \
    --fp $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp --cggtts -c $CONF
