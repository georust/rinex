#!/bin/bash
DATA_DIR=test_resources

SYSTEM=GPS
# SYSTEM=G13,G15,G24,G28

TIMEFRAME=">2020-06-25T00:00:00 GPST"

CONF=config/survey/cpp_lsq.json
CONF=config/survey/cpp_kf.json
CONF=config/survey/cpp_lsq.json

./target/release/rinex-cli \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -f $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    -f $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    -P $SYSTEM -P "$TIMEFRAME" -p -c $CONF | tee logs/esbjr-gps+cpp.txt
