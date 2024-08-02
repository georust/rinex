#!/bin/sh
# Post processed (+3week) high precision surveying
# of ESBC00DNK station, using BeiDou constellation
# Change the configuration scripts to modify navigation parameters.
# Remove either step to remove that chapter from the synthesized report.
DATA_DIR=test_resources

# filter: All BeiDou
#  when doing this, the report is split between BeiDou (GEO) and BeiDou (MEO)
#  Refer to BDS-GEO or similar, for other examples
FILTER=BeiDou

# Custom surveying config
CONF=tutorials/config/survey/cpp_lsq.json

# Analysis + ppp solutions
# -q: silent: open on very last run only
# -f: force new synthesis
# -P: filter example
./target/release/rinex-cli \
    -P $FILTER -f -q \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/Sta21114.sp3.gz \
    ppp -c $CONF

# cggtts solutions (+open).
# Since we're using strict identical options,
# the report is preserved and new solutions are appended.
./target/release/rinex-cli \
    -P $FILTER \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/Sta21114.sp3.gz \
    ppp --cggtts -c $CONF
