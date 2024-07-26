#!/bin/sh

# Although CGGTTS is a laboratory/advanced technique,
# we can deploy it on any data sources.
# With this example, we can produce one CGGTTS for remote comparison of the clock embedded
# in the mobile phone.
DATA_DIR=test_resources

#  GPS : any GPS; CGGTTS is feasible on any constellation
# L1+L5: CPP
# SV   : CGGTTS allows single SV common view clock comparison
#        it is a lengthy process, you usually want to resolve for SV
#        that are seen both sites
FILTER="GPS;C1C,C5Q;G06,G11,G24,G28,G29" # all GPS

# Tracking:
#   We only have 10' of sky observations, which is incompatible with standard CGGTTS (laboraty technique).
#   So we need to reduce the duration of the CGGTTS tracks.
#   Sampling interval is 1s, the tracks will use 60 data pts.
TRACKING="1 min"

# CGGTTS opmode has other options to customize the solution.
# With this, we define the custom data producer ("agency"). 
PRODUCER=JMF

# CGGTTS opmode has other options to customize the solution.
# With this, we identify the local clock.
CLOCK=JMF-ANDROID

# Any strategy may apply to CGGTTS,
# it will just modify and possibly improve the solutions
CONF=config/survey/spp_lsq.json
CONF=config/survey/cpp_kf.json

OBS=test_resources/OBS/V3/GEOP092I.24o.gz
SP3=test_resources/SP3/GFZ0OPSRAP_20240920000_01D_05M_CLK.CLK.gz
CLK=test_resources/CLK/V3/GFZ0OPSRAP_20240920000_01D_05M_ORB.SP3.gz
NAV=test_resources/NAV/V3/HERT00GBR_R_20240920000_01D_GN.rnx.gz

./target/release/rinex-cli \
    -P $FILTER \
    -f $OBS -f $NAV -f $SP3 -f $CLK \
    ppp -c $CONF \
    --cggtts --lab $PRODUCER --clk $CLOCK -t "$TRACKING" \
    | tee logs/jmf-24092+cggtts.txt
