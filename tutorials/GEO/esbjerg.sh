#!/bin/sh
# Post processing:  
#  +3 week surveying of this lab station
# Compare this to real time (BRDC)

# Comment out one step to remove it from the synthesized report.
# Change the configuration to modify the navigation setup.
DATA_DIR=test_resources

# Example:
#   GPS <28 : any other is dropped
#  L1/L5 PR only: not using L2, not PPP compatible
FILTER="GEO"
CONF=tutorials/config/survey/cpp_kf.json

# Analysis + ppp solutions (silent)
#  -f: force new synthesis
#  -q: open on last run only
./target/release/rinex-cli \
    -P $FILTER -f \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp -c $CONF
