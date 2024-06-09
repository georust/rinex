#!/bin/sh
DATA_DIR=test_resources
SYSTEM=Gal # all Galileo
CONF=config/survey/spp_lsq.json # =pseudorange;LSQ

./target/release/rinex-cli \
   -f $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
   -f $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
   -P $SYSTEM -p -c $CONF | tee logs/mojn-gal-spp+brdc.txt
