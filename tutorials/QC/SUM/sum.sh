#!/bin/sh
# PPP context
# Adjust the input products to modify reported content
DATA_DIR=test_resources

# Example: 
#  Gal      PRN>09
#  GPS      PRN>=10
#  BeiDou   PRN>15
# Signals   B2I, E1, L1
FILTER="Gal,GPS;C2I,C1C"
# Skip 1st hour (example)
TIMEFRAME=">2020-06-25T01:00:00 UTC"

# ppp-like context analysis
#  -f: force report synthesis
#  -P: filter example
./target/release/rinex-cli \
    -f -o "GpsGalC2iC1c-QC" \
    -P $FILTER "$TIMEFRAME" \
    --fp $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz
