#! /bin/sh
# Post Processed (+3 week SP3/CLK) analysis (only)

OBS=test_resources/OBS/V3/GEOP092I.24o.gz
SP3=test_resources/SP3/GFZ0OPSRAP_20240920000_01D_05M_ORB.SP3.gz
CLK=test_resources/CLK/V3/GFZ0OPSRAP_20240920000_01D_05M_CLK.CLK.gz
NAV=test_resources/NAV/V3/BRUX00BEL_R_20240920000_01D_EN.rnx.gz

./target/release/rinex-cli \
    -P Galileo \
    --fp $OBS \
    --fp $NAV \
    --fp $SP3 \
    --fp $CLK
