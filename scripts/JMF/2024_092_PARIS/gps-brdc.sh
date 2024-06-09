#! /bin/sh
# "Real-time" (BRDC) surveying
# GPS performances are compatible with our default scripts
CONF=config/survey/cpp_kf.json

OBS=test_resources/OBS/V3/GEOP092I.24o.gz
NAV=test_resources/NAV/V3/HERT00GBR_R_20240920000_01D_GN.rnx.gz

./target/release/rinex-cli \
    -f $OBS -f $NAV \
    -P GPS \
    -p -c $CONF | tee logs/jmf-24092+gps+brdc.txt
