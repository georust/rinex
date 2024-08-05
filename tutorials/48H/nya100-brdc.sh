#!/bin/sh
# Real time surveying (BRDC) using 48h of observations from NYA100 (arctic).
# Some channels have zero (null) values - which is illegal: use -z for repair.
DATA_DIR=test_resources
CONF=tutorials/config/survey/cpp_kf.json

# Load two sets of observations from the same site (order never matters)
# The observation reports show that the observation period is continuous (not a single gap!)
# This totalizes 5000+ epochs we can then accumulate.
# We add ephemerides for both days, gathered by the same site but any local area may apply (NYA200, ...)

# PPP solutions (ppp) with single Signal=C1X
# -f: force report synthesis
# -q: silent reporting
# -z: this site tends to publish zero values on X channel (I+Q) 
./target/release/rinex-cli \
    -P "Gal;C1X,C5X" \
    -f -z -q -o "Gal48h-L1L5" \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241270000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241280000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241270000_01D_EN.rnx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241280000_01D_EN.rnx.gz \
    ppp -c $CONF

# add CGGTTs solutions +open
./target/release/rinex-cli \
    -P "Gal;C1X,C5X" \
    -f -z -q -o "Gal48h-L1L5" \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241270000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241280000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241270000_01D_EN.rnx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241280000_01D_EN.rnx.gz \
    ppp --cggtts -c $CONF
