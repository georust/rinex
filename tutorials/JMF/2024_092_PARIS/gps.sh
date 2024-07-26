#! /bin/sh
# Post Processed (+3 week SP3/CLK) surveying
# GPS performances are compatible with our default scripts
CONF=tutorials/config/survey/cpp_kf.json

OBS=test_resources/OBS/V3/GEOP092I.24o.gz
SP3=test_resources/SP3/GFZ0OPSRAP_20240920000_01D_05M_CLK.CLK.gz
CLK=test_resources/CLK/V3/GFZ0OPSRAP_20240920000_01D_05M_ORB.SP3.gz
NAV=test_resources/NAV/V3/HERT00GBR_R_20240920000_01D_GN.rnx.gz

./target/release/rinex-cli \
    -P GPS \
    --fp $OBS \
    --fp $NAV \
    --fp $SP3 \
    --fp $CLK \
    ppp -c $CONF
