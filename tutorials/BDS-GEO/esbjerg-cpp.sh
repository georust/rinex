#!/bin/sh
DATA_DIR=test_resources
CONF=config/survey/cpp_lsq.json
SYSTEM=BeiDou # All BeiDou

./target/release/rinex-cli \
    -f $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -f $DATA_DIR/SP3/Sta21114.sp3.gz \
    -P $SYSTEM -p -c $CONF | tee logs/esbjr-bds+spp+brdc.txt
