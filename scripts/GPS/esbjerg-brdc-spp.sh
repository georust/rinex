#!/bin/sh
DATA_DIR=test_resources
SYSTEM=GPS # all GPS
CONF=config/survey/spp_lsq.json # pseudorange + LSQ

./target/release/rinex-cli \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -P $SYSTEM -p -c $CONF | tee logs/esbjr-spp+brdc.txt
