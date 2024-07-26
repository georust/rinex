#!/bin/sh
# Real time surveying (BRDC) using Pseudo Range
DATA_DIR=test_resources
# Example: E1 All Galileo
FILTER="Gal;C1C"
CONF=tutorials/config/survey/spp_lsq.json # =pseudorange;LSQ

./target/release/rinex-cli \
   -P $FILTER \
   --fp $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
   --fp $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
   ppp -c $CONF
