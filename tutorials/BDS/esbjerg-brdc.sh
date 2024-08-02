#!/bin/sh
DATA_DIR=test_resources
CONF=tutorials/config/survey/cpp_kf.json

SYSTEM=BeiDou # All BeiDou
#SIGNALS=C2I,C6I
SIGNALS=C2I,C7I

# Analysis + ppp solutions
#  -q: silent (open only on last run)
#  -f: force new synthesis
#  -P: filter example
./target/release/rinex-cli \
    -q -f \
    -P $SYSTEM -P $SIGNALS \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp -c $CONF

# append cggtts solutions +open
./target/release/rinex-cli \
    -P $SYSTEM -P $SIGNALS \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp --cggtts -c $CONF
