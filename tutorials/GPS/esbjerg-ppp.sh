#!/bin/sh
# Post processed (+3week) PPP surveying using GPS=L1/L2
DATA_DIR=test_resources
# Example:
#   GPS <30 : any other are dropped
#  L1/L2only: not using L5
FILTER="GPS;<G30;C1C,C2W,L1C,L2W"
# Example: skip first hour of that day
TIMEFRAME=">2020-06-25T01:00:00 GPST"
CONF=config/survey/ppp_kf.json # PPP;filter=KF;x17

./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -f $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    -f $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp -c $CONF | tee logs/esbjr-gps+ppp.txt
