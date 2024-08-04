#! /bin/sh
# Post Processed (+3 week SP3/CLK) surveying
# GPS performances are compatible with our default scripts.
# Modify the config script to customize navigation parameters.
CONF=tutorials/config/survey/cpp_kf.json

## full ppp like context
OBS=test_resources/OBS/V3/GEOP092I.24o.gz
SP3=test_resources/SP3/GFZ0OPSRAP_20240920000_01D_05M_CLK.CLK.gz
CLK=test_resources/CLK/V3/GFZ0OPSRAP_20240920000_01D_05M_ORB.SP3.gz
NAV=test_resources/NAV/V3/HERT00GBR_R_20240920000_01D_GN.rnx.gz

# ppp solutions 
#  -f: force new synthesis
#  -q: silent (open on last cggtts run)
./target/release/rinex-cli \
    -P GPS \
    -f -q -o "GPS-L1L5" \
    --fp $OBS \
    --fp $NAV \
    --fp $SP3 \
    --fp $CLK \
    ppp -c $CONF

# cggtts solutions (+open)
# When using exact same input parameters: report is preserved
# and new chapters are appended.
# Although CGGTTS is a laboraty/advanced technique,
# we can deploy it on any context (this is just an example).

# Such a short observation period is incompatible with standard cggtts,
# we need to move on to special tracking.
TRACKING="1 min"

# customization example
PRODUCER="JMF"
CLOCK="ANDROID"

# cggtts solutions
./target/release/rinex-cli \
    -P GPS \
    --fp $OBS \
    --fp $NAV \
    --fp $SP3 \
    --fp $CLK \
    ppp --cggtts -c $CONF \
    --lab $PRODUCER --clk $CLOCK --trk "$TRACKING"
