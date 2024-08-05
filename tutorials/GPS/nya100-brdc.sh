#!/bin/sh
# Real time surveying (BRDC) using observations from NYA100 (arctic).
# Some channels have zero (null) values - which is illegal, we work it around.
# NYA100 & NYA200 are located on the same site.
#  Therefore, they share identical conditions, for example we can share NAV/V3 datasets.
DATA_DIR=test_resources

# Force new report (-f)
# Silent reporting (-q): open on last run only
# PPP solutions (ppp) with single Signal=C1C
./target/release/rinex-cli \
    -P "GPS;C1C" \
    -f -q -o "GPS-L1" \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_GN.rnx.gz \
    ppp -c tutorials/config/survey/spp_lsq.json

# Append PPP solutions (ppp) with single signal=C2X
# Zero repair (-z): their X (=I+Q) channels have zero values
#      which is illegal!! and causes the solver to diverge (eventually panic).
#      simply remove them :)
./target/release/rinex-cli \
    -P "GPS;C2X" \
    -f -q -z -o "GPS-L2" \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_GN.rnx.gz \
    ppp -c tutorials/config/survey/spp_lsq.json

# Dual channel Navigation (C1C+C2X) compare to past runs
# Zero repair (-z): their X (=I+Q) channels have zero values
#      which is illegal!! and causes the solver to diverge (eventually panic).
#      simply remove them :)
./target/release/rinex-cli \
    -P "GPS;C1C,C2X" \
    -f -q -z -o "GPS-L1L2" \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_GN.rnx.gz \
    ppp -c tutorials/config/survey/cpp_kf.json

# Generate CGGTTS solutions for the dual channel run as well.
# Open report.
# Zero repair (-z): their X (=I+Q) channels have zero values
#      which is illegal!! and causes the solver to diverge (eventually panic).
#      simply remove them :)
./target/release/rinex-cli \
    -P "GPS;C1C,C2X" \
    -z -o "GPS-L1L2" \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_GN.rnx.gz \
    ppp --cggtts -c tutorials/config/survey/cpp_kf.json
