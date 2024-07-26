#!/bin/sh
# BRDC navigation quality control
DATA_DIR=test_resources

# Report automatically adapts to provided context
# This is a BRDC NAV example, refer to the MOJDNK example for PPP like context
OBS=$DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
NAV=$DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \

# Example:
#   GPS >09 && <26 any other constellation and PRN
#            will not be included to the report
#  Skip last hour of that day (example)
FILTER="GPS;>G09;<G26;"
TIMEFRAME="<2020-06-25T23:00:00 UTC"

./target/release/rinex-cli \
    -P $FILTER -P "$TIMEFRAME" \
    --fp $OBS \
    --fp $NAV
