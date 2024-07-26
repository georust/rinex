#! /bin/sh
# "Real-time" (BRDC) surveying

# Custom config: GDOP we're obtaining from Galileo is incompatible
#   with our default config/survey configs (Mobile Phone?)
# CPP; x17; 
# filter=kalman;
# System=GPST
# GDOP=200
CONF=scripts/JMF/2024_092_PARIS/cpp_kf.json

OBS=test_resources/OBS/V3/GEOP092I.24o.gz
NAV=test_resources/NAV/V3/BRUX00BEL_R_20240920000_01D_EN.rnx.gz
    
./target/release/rinex-cli \
    -P Gal \
    -f $OBS -f $NAV \
    ppp -c $CONF | tee logs/jmf-24092-gal+brdc.txt
