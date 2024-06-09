#!/bin/sh
DATA_DIR=test_resources
CONF=config/survey/ppp_kf.json
SYSTEM=GPS # all GPS
SIGNALS=C1C,C5Q,L1C,L5Q # L1/L5 PR+PH

# TIMEFRAME=">2020-06-25T12:00:00 GPST"
# TIMEFRAME=">2020-06-25T05:30:00 GPST"
TIMEFRAME=">2020-06-25T00:00:00 GPST" # entire timeframe; therefore, could be omitted

./target/release/rinex-cli \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -f $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    -f $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    -P $SYSTEM -P $SIGNALS -P "$TIMEFRAME" -p -c $CONF | tee logs/esbjr-gps+ppp.txt
