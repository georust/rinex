#!/bin/sh
# Surveying with Galileo and 1 GEO

# In this example, we use signals E1+E5(PR), but that does not matter.
# We use Galileo and 1 geostationnary vehicle.
# By using 4 GEO vehicles are observed (see other analysis examples)
# By using >S35 in the filter, we make sure that only one is kept.
# This is BRDC nav ("real time") surveying augmented with one GEO.
# Compare this one to esbjerg-ppp-x1 where we move to precise post processing, augmented with one GEO.
# Comare also to esbjerg-brdc and esbjerg-precise (in this very folder) when use GEO and GAL in equal proportion

DATA_DIR=test_resources
FILTER="Gal,GEO;<E05;C1C,C5Q" 
CONF=tutorials/config/survey/cpp_kf.json

./target/release/rinex-cli \
    -f -o "GAL+SBAS-E1E5" \
    -P $FILTER \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    ppp -c $CONF

exit 0
    #--fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    #--fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
./target/release/rinex-cli \
    -P $FILTER \
    --fp $DATA_DIR/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp $DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    --fp $DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    ppp --cggtts -c $CONF
