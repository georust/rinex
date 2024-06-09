#!/bin/sh
DATA_DIR=test_resources

SYSTEM=Gal # all galileo
SIGNALS=C1C,C5Q,L1C,L5Q # E1/E5 Phase+Code
CONF=config/survey/ppp_kf.json # Phase+Code;filter=kalman;x17

./target/release/rinex-cli \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -f $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    -f $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    -P $SYSTEM -P $SIGNALS -p -c $CONF | tee logs/esbjr-gal+ppp.txt
