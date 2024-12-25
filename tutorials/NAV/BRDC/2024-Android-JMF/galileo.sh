#! /bin/sh
# Post Processed (+3 week SP3/CLK) surveying

# Custom config: GDOP we're obtaining from Galileo is incompatible
#   with our default config/survey configs (Mobile Phone?)
# CPP; x17; 
# filter=kalman;
# System=GPST
# GDOP=200
CONF=tutorials/JMF/2024_092_PARIS/cpp_kf.json
    
## full ppp like context
OBS=test_resources/OBS/V3/GEOP092I.24o.gz
SP3=test_resources/SP3/GFZ0OPSRAP_20240920000_01D_05M_ORB.SP3.gz
CLK=test_resources/CLK/V3/GFZ0OPSRAP_20240920000_01D_05M_CLK.CLK.gz
NAV=test_resources/NAV/V3/BRUX00BEL_R_20240920000_01D_EN.rnx.gz

# ppp solutions
#  -f: force new report
#  -q: quiet (open on cggtts last run)
./target/release/rinex-cli \
    -f -q -o "Gal-E1E5" \
    -P Galileo \
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

# cggtts solutions (+open)
./target/release/rinex-cli \
    -P Galileo \
    --fp $OBS \
    --fp $NAV \
    --fp $SP3 \
    --fp $CLK \
    ppp --cggtts -c $CONF \
    --lab $PRODUCER --clk $CLOCK --trk "$TRACKING"
