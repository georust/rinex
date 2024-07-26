#!/bin/sh
# Post processed (+3 week) surveying of the Esbjerg (DNK) station clock to GST
DATA_DIR=test_resources
SYSTEM=GPS # all GPS

# Any strategy may apply to CGGTTS,
# it will just modify and possibly improve the solutions
CONF=tutorials/config/survey/spp_lsq.json
CONF=tutorials/config/survey/cpp_kf.json

# Any Pseudo Range: remove one to reduce the possible solutions
SIGNALS=C1C,C1W,C2L,C2W,C5Q

# CGGTTS allows single SV common view time transfer
# and is also a time consuming process,
# usually we want to resolve for very specific SV that are seen on both sites.
SV=G05,G08,G15,G24 # This is just an example, remove this to resolve for all SV

./target/release/rinex-cli \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    -P $SYSTEM \
    -P $SIGNALS \
    -P $SV \
    ppp -c $CONF --cggtts
