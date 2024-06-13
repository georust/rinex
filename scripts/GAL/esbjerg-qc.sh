#!/bin/sh
# Real time surveying (BRDC) using 1D Pseudo Range LSQ
DATA_DIR=test_resources
CONF=config/qc/spp_lsq.json

# analysis over all Galileo
SYSTEM=Gal

./target/release/rinex-cli \
    -P Gal \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    qc
