#!/bin/sh
# Post processed (+3 week) surveying of the MOJN (DNK) lab station
DATA_DIR=test_resources
# Example: 
#  Gal PRN>09
#  Skip 1st hour
#  E1 or E5 PR
FILTER="Gal,GEO;>E08;C1C,C5Q"
# Skip 1st hour (example)
TIMEFRAME=">2020-06-25T01:00:00 UTC"
CONF=tutorials/config/survey/cpp_kf.json # pseudo-range(=e1,e5); filter:kalman

./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    --fp $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp -c $CONF

./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    --fp $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp --cggtts -c $CONF
