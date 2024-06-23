#!/bin/sh
# PPP quality control (data is post processed +3 weeks)
#  See Esbjrg example for BRDC example.
DATA_DIR=test_resources

# Report automatically adapts to provided context
OBS=$DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz
SP3=$DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz
NAV=$DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz
CLK=$DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz

# Example:
#   Gal >09
#   BDS (non GEO)
FILTER="Gal,BDS;>E09;>C05;<C58"

./target/release/rinex-cli \
    -P "$FILTER" \
    -f $OBS \
    qc
