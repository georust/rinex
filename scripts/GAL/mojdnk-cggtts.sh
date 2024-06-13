#!/bin/sh
# Post processed (+3 week) surveying of the MOJN (DNK) station clock to GST
DATA_DIR=test_resources

# Any strategy may apply to CGGTTS,
# it will just modify and possibly improve the solutions
CONF=config/survey/spp_lsq.json # basic SPP+LSQ
CONF=config/survey/cpp_kf.json # basic CPP+KF

# Example:
#  Gal >05;<20  CGGTTS allows single SV common view clock comparison
#               it is a slower process because it requires more iteration
#               usually you want to resolve for specific SV only (seen on both sites)
# Signals: L1+L5 (example)
# Timeframe: skip two hours (example)
FILTER="Gal;>E05;<E20;C1C,C1W,C5Q;"
TIMEFRAME=">2020-06-25T02:00:00 UTC"

./target/release/rinex-cli \
    -P $FILTER "$TIMEFRAME" \
    -f $DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz \
    -f $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    -f $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp -c $CONF --cggtts | tee logs/mojn-gal+cggtts.txt
