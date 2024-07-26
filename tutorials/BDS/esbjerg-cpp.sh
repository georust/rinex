#!/bin/sh
# Post processed (+3week) high precision surveying
# of ESBC00DNK station, using BeiDou constellation
DATA_DIR=test_resources

# filter: All BeiDou
#  when doing this, the report is split between BeiDou (GEO) and BeiDou (MEO)
#  Refer to BDS-GEO or similar, for other examples
FILTER=BeiDou

# Custom surveying config
CONF=tutorials/config/survey/cpp_lsq.json

./target/release/rinex-cli \
    -P $FILTER \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/Sta21114.sp3.gz \
    ppp -c $CONF
