#!/bin/sh
DATA_DIR=test_resources
SCRIPTS_DIR=scripts
CONF=tutorials/config/survey/spp_lsq.json

./target/release/rinex-cli \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -P BDS -p -c $CONF | tee logs/esbjr-bds+spp+brdc.txt
