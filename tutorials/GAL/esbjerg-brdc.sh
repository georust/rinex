#!/bin/sh
# Real time surveying (BRDC) using Pseudo Range
DATA_DIR=test_resources
# In this example, we consider all Gal vehicles
SYSTEM=Gal
CONF=tutorials/config/survey/spp_lsq.json # basic SPP conf

./target/release/rinex-cli \
    -P Gal \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp -c $CONF