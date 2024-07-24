#!/bin/sh
DATA_DIR=test_resources
CONF=config/survey/cpp_kf.json

SYSTEM=BeiDou # All BeiDou
#SIGNALS=C2I,C6I
SIGNALS=C2I,C7I

./target/release/rinex-cli \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -P $SYSTEM -P $SIGNALS -p -c $CONF | tee logs/esbjr-bds-cpp+brdc.txt
