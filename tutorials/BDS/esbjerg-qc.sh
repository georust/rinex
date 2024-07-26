#!/bin/sh
# BRDC navigation quality control
#  See MOJDNK example for PPP example.
DATA_DIR=test_resources

# Report automatically adapts to provided context
# This is a BRDC NAV example, refer to the MOJDNK example for PPP like context
OBS=$DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
NAV=$DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz

# Example:
#   BDS >10 && <26 Non GEO specific filter
#  Skip last hour of that day (example)
FILTER="Galileo;>E09;<E26"
TIMEFRAME="<2020-06-25T23:00:00 UTC"

./target/release/rinex-cli \
    -P $FILTER \
    -P "$TIMEFRAME" \
    --fp $OBS \
    --fp $NAV
