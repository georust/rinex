#!/bin/sh
DATA_DIR=test_resources

SYSTEM=Galileo # all Gal
TIMEFRAME='>2020-06-25T00:00:00 UTC' #=entire day; therefore, could be omitted
SIGNALS=C1C,C5Q # E1;E5 pseudo range
CONF=config/survey/cpp_kf.json # pseudo-range(e1/e5); filter:kalman

./target/release/rinex-cli \
   -f $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
   -f $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
   -f $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
   -f $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
   -P $SYSTEM -P $SIGNALS -P "$TIMEFRAME" -p -c $CONF | tee logs/mojn-gal+cpp.txt
