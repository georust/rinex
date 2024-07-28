#!/bin/sh
# In this example, we only keep the GEO vehicles and run their analysis.
# The context is PPP like.
# You can refer to MOJNDNK-GEO for similar but PPP like navigation.
# You can refer to scripts/BDS/* for the opposite filter example.
DATA_DIR=test_resources

# Report automatically adapts to provided context
# This is a PPP-ultra compatible context, simply remove one file to change that.
OBS=$DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz
SP3=$DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz
NAV=$DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz
CLK=$DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz

# Example:
#  Preserve GEO only
#  Skip last hour of that day (example)
FILTER="BeiDou;<C06;>C55;<2020-06-25T23:00:00 UTC"

./target/release/rinex-cli \
    -P $FILTER \
    -P "$TIMEFRAME" \
    --fp $OBS \
    --fp $SP3 \
    --fp $NAV \
    --fp $CLK
