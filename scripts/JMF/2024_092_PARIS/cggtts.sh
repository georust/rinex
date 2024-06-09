#!/bin/sh

# Although CGGTTS is a laboratory/advanced technique,
# we can deploy it on any data sources.
# With this example, we can produce one CGGTTS for remote comparison of the clock embedded
# in the mobile phone.
DATA_DIR=test_resources

# CGGTTS is feasible on any constellation, this just a GPS example.
# You can rework the Galileo example to deploy CGGTTS on this data too.
SYSTEM=GPS # all GPS

# We only have 10' of sky observations, which is incompatible with standard CGGTTS (laboraty technique).
# So we need to reduce the duration of the CGGTTS tracks.
# We used a 1s observation interval, the tracks will use 60 pts.
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

# Any Pseudo Range: remove one to reduce the possible solutions
SIGNALS=C1C,C5Q

# CGGTTS allows single SV common view time transfer
# and is also a time consuming process,
# usually we want to resolve for very specific SV that are seen on both sites.
SV=G06,G11,G12,G24,G28,G29 # This is just an example, remove this to resolve for all SV

OBS=test_resources/OBS/V3/GEOP092I.24o.gz
SP3=test_resources/SP3/GFZ0OPSRAP_20240920000_01D_05M_CLK.CLK.gz
CLK=test_resources/CLK/V3/GFZ0OPSRAP_20240920000_01D_05M_ORB.SP3.gz
NAV=test_resources/NAV/V3/HERT00GBR_R_20240920000_01D_GN.rnx.gz

./target/release/rinex-cli \
    -f $OBS -f $NAV -f $SP3 -f $CLK \
    -P $SYSTEM \
    -P $SIGNALS \
    -P $SV \
    -p -c $CONF \
    --cggtts --lab $PRODUCER --clk $CLOCK -t "$TRACKING" \
    | tee logs/jmf-24092+cggtts.txt
