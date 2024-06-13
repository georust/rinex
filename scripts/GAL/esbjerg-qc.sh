#!/bin/sh
# BRDC navigation quality control
#  See MOJDNK example for PPP example.
DATA_DIR=test_resources

# Report automatically adapts to provided context
OBS=$DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz
NAV=$DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \

# Example:
#   Gal >10 && <26 any other constellation and PRN
#            will not be included to the report
#  Skip last hour of that day (example)
FILTER="Galileo;>E09;<E26;<2020-06-25T23:00:00 UTC"

./target/release/rinex-cli \
    -P $FILTER \
    -f $OBS -f $NAV \
    qc
