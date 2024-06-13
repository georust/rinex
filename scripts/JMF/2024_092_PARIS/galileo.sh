#! /bin/sh
# Post Processed (+3 week SP3/CLK) surveying

# Custom config: GDOP we're obtaining from Galileo is incompatible
#   with our default config/survey configs (Mobile Phone?)
# CPP; x17; 
# filter=kalman;
# System=GPST
# GDOP=200
CONF=scripts/JMF/2024_092_PARIS/cpp_kf.json
    
OBS=test_resources/OBS/V3/GEOP092I.24o.gz
SP3=test_resources/SP3/GFZ0OPSRAP_20240920000_01D_05M_ORB.SP3.gz
CLK=test_resources/CLK/V3/GFZ0OPSRAP_20240920000_01D_05M_CLK.CLK.gz
NAV=test_resources/NAV/V3/BRUX00BEL_R_20240920000_01D_EN.rnx.gz

./target/release/rinex-cli \
    -P Galileo \
    -f $OBS -f $NAV -f $SP3 -f $CLK \
    ppp -c $CONF | tee logs/jmf-24092+gal.txt
