#!/bin/sh
# Loading a single SP3 will generate Orbital projections
# If you stack one BRDC file, you can then run orbital state comparison
DATA_DIR=test_resources

# Example: 
#  Gal      PRN>09
#  GPS      PRN>=10
FILTER="GPS"
# With SP3 only, no position preset is defined.
# We need to manually specifiy one. Attitudes will be projected
# with respect to that preset. This is usually the GNSS antenna location.
RX_ECEF="3582105.2910,532589.7313,5232754.8054"

#  -f: force report synthesis
#  -P: filter example
./target/release/rinex-cli \
    -f -q \
    --rx-ecef $RX_ECEF \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz
