#!/bin/sh
# Post processed (+3 week) surveying of the Esbjerg (DNK) lab station
DATA_DIR=test_resources
# E1/E5 PPP for Gal+PRN>08 (Example)
FILTER="Gal;>=E09;C1C,C5Q,L1C,L5Q"
CONF=config/survey/ppp_kf.json # PPP;filter=kalman;x17

./target/release/rinex-cli \
    -P $FILTER \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp -c $CONF | tee logs/esbjr-gal+ppp.txt
