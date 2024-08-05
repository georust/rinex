#!/bin/sh
# Real time surveying (BRDC) using 48h of observations from AJACFRA (Corsica/fr).
# This station does not publish ephemerides, we use ephemerides published by nearby site.
DATA_DIR=test_resources
CONF=tutorials/config/survey/cpp_kf.json

# Load two sets of observations from the same site (order never matters)
# The observation reports show that the observation period is continuous (not a single gap!)
# This totalizes 5000+ epochs we can then accumulate.
# We add ephemerides for both days, gathered by the same site but any local area may apply

# PPP solutions (ppp) with single Signal=C1C/C5Q
# -f: force report synthesis
# -q: silent reporting
./target/release/rinex-cli \
    -P "Gal;C1C,C5Q" \
    -f -z -q -o "Gal48h-L1L5" \
    --fp $DATA_DIR/CRNX/V3/AJAC00FRA_R_20242090000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/CRNX/V3/AJAC00FRA_R_20242100000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/GRAS00FRA_R_20242090000_01D_EN.rnx.gz \
    --fp $DATA_DIR/NAV/V3/GRAS00FRA_R_20242100000_01D_EN.rnx.gz \
    ppp -c $CONF
