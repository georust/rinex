#!/bin/sh
# Real time survey (BRDC)
# Compare this to post processed (+3weeks)

# Comment out one step to remove it from the synthesized report.
# Change the configuration to modify the navigation setup.
DATA_DIR=test_resources

# Example: 
#  GPS PRN>09
#  L1 or L5 PR
FILTER="GPS;>G09;C1C,C5Q"
CONF=tutorials/config/survey/cpp_kf.json # pseudo-range(L1/L5); filter:kalman

# Analysis + ppp solutions (silent)
#   -f: force new report synthesis
#   -q: silent (open on last call)
#   -o: custom name
./target/release/rinex-cli \
    -P $FILTER \
    -f -q -o "BRDC-GPS-L1L5" \
    --fp $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp -c $CONF
exit 0

# cggtts solutions (+open).
# Since we're using strict identical options,
# the report is preserved and new solutions are appended.
# The report is automatically opened.
./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    -o "BRDC-GPS-L1L5" \
    --fp $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp --cggtts -c $CONF
