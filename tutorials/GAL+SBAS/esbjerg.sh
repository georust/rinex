#!/bin/sh
# Post processing:  
#  +3 week surveying of this lab station

DATA_DIR=test_resources
# In this example, we use signals E1+E5(PR) for Galileo SV PRN>08 (example)
#  and one EGNOS vehicle. You should compare these results to tutorial/GAL/esbcdnk-cpp.sh
FILTER="Gal,GEO;>E08;C1C,C5Q" 
CONF=tutorials/config/survey/cpp_kf.json # basic CPP+Kf

./target/release/rinex-cli \
    -P $FILTER \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp -c $CONF

./target/release/rinex-cli \
    -P $FILTER \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp --cggtts -c $CONF
