#!/bin/bash
DATA_DIR=test_resources

SYSTEM=Galileo # all Gal
TIMEFRAME='>2020-06-25T00:00:00 UTC' # entire day
SIGNALS=C1C,C5Q

# Strategy: 
# NAV: Kalman
# Other: X17(SP3) + Eclipse filter
CONF=config/survey/cpp_lsq.json
CONF=config/survey/cpp_kf_eclipse.json

./target/release/rinex-cli \
   -f $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
   -f $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
   -f $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
   -f $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
   -P $SYSTEM -P $SIGNALS -P "$TIMEFRAME" -p -c $CONF | tee logs/mojn-gal.txt
