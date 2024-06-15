#!/bin/sh
# PPP quality control (data is post processed +3 weeks)
#  See Esbjrg example for BRDC example.
DATA_DIR=test_resources

# Report automatically adapts to provided context
OBS=$DATA_DIR/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz
SP3=$DATA_DIR/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz
NAV=$DATA_DIR/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz
CLK=$DATA_DIR/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz
METEO=$DATA_DIR/MET/V3/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz

# Example:
#   Gal >09: any other constellation and PRN
#            will not be included to the report
#  Skip first hour of that day (example)
FILTER="Galileo;>E09;>2020-06-25T00:00:00 UTC"

./target/release/rinex-cli \
    -P $FILTER \
    -f $OBS -f $SP3 -f $CLK -f $NAV -f $METEO \
    qc
