#!/bin/sh
# Post processed (+3 week) surveying of the Esbjerg (DNK) lab station
DATA_DIR=test_resources
# Example: E1(PR) for Galileo SV PRN>14
FILTER="Gal;>E14;C1C"
CONF=config/survey/spp_lsq.json # Basic SPP;filter=LSQ

./target/release/rinex-cli \
    -P $FILTER \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -f $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    -f $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp -c $CONF | tee logs/gal-esbjrg-spp+lsq.txt
