#!/bin/sh
# Post processed (+3 week) surveying of the MOJN (DNK) lab station
DATA_DIR=test_resources
# Example: 
#  GPS PRN>05
#  Skip 1st hour
#  L1/L5
FILTER="GPS;>G05;"
# Skip 1st hour (example)
TIMEFRAME=">2020-06-25T01:00:00 UTC"
CONF=config/survey/ppp_kf.json

./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    -f $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    -f $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    -f $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp -c $CONF | tee logs/mojn-gps+ppp.txt
