#!/bin/sh
# Real time surveying (BRDC) using Pseudo Range
DATA_DIR=test_resources
# Example: E1 All Galileo
FILTER="Gal;C1C"
CONF=config/survey/spp_lsq.json # =pseudorange;LSQ

./target/release/rinex-cli \
   -P $FILTER \
   -f $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
   -f $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
   ppp -c $CONF | tee logs/mojn-gal-spp+brdc.txt
