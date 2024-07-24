#!/bin/sh
# In this example, we only keep the GEO vehicles and run their analysis.
# The context is BRDC like navigation.
# You can refer to MOJNDNK-GEO for similar but PPP like navigation.
# You can refer to scripts/BDS/* for the opposite filter example.
DATA_DIR=test_resources

# Report automatically adapts to provided context
OBS=$DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
NAV=$DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz

# Example:
#  Preserve GEO only
#  Skip last hour of that day (example)
FILTER="BeiDou;<C30;>C55"
TIMEFRAME="<2020-06-25T23:00:00 UTC"

./target/release/rinex-cli \
    -P $FILTER \
    -P "$TIMEFRAME" \
    --fp $OBS \
    --fp $NAV
