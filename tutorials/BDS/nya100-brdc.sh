#!/bin/sh
# Real time surveying (BRDC)
DATA_DIR=test_resources

# Comment out one step to remove it from the synthesized report.
# Change the configuration to modify the navigation setup.
DATA_DIR=test_resources

FILTER="BDS"
CONF=tutorials/config/survey/spp_lsq.json

# Analysis + ppp solutions (silent)
./target/release/rinex-cli \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_CN.rnx.gz

 exit 0

# cggtts solutions (+open).
# Since we're using strict identical options,
# the report is preserved and new solutions are appended.
# The report is automatically opened.
./target/release/rinex-cli \
    -P $FILTER \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_GN.rnx.gz \
    ppp --cggtts -c $CONF
