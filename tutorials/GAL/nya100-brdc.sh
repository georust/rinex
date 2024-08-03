#!/bin/sh
# Real time surveying (BRDC) using observations from NYA100 (arctic).
# Some channels have zero (null) values - which is illegal, we work it around.
DATA_DIR=test_resources

# Force new report (-f)
# Silent reporting (-q): open on last run only
# Zero repair (-z): their X (=I+Q) channels have zero values
#      which is illegal!! and causes the solver to diverge (eventually panic).
#      simply remove them :)
# PPP solutions (ppp) with single Signal=C1X
./target/release/rinex-cli \
    -P "Gal;C1X" \
    -f -z -q -o "Gal-L1i+q" \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_EN.rnx.gz \
    ppp -c tutorials/config/survey/spp_lsq.json

# Append PPP solutions (ppp) with single signal=C5X
# Zero repair (-z): their X (=I+Q) channels have zero values
#      which is illegal!! and causes the solver to diverge (eventually panic).
#      simply remove them :)
./target/release/rinex-cli \
    -P "Gal;C5X" \
    -f -q -z -o "Gal-L5i+q" \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_EN.rnx.gz \
    ppp -c tutorials/config/survey/spp_lsq.json

# Dual channel Navigation (C1C+C5X) compare to past runs
# Zero repair (-z): their X (=I+Q) channels have zero values
#      which is illegal!! and causes the solver to diverge (eventually panic).
#      simply remove them :)
./target/release/rinex-cli \
    -P "Gal;C1X,C5X" \
    -f -q -z -o "Gal-L1i+qL5i+q" \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_EN.rnx.gz \
    ppp -c tutorials/config/survey/cpp_kf.json

# Generate CGGTTS solutions for past run as well.
# Open report.
# Zero repair (-z): their X (=I+Q) channels have zero values
#      which is illegal!! and causes the solver to diverge (eventually panic).
#      simply remove them :)
./target/release/rinex-cli \
    -P "Gal;C1X,C5X" \
    -z -o "Gal-L1i+qL5i+q" \
    --fp $DATA_DIR/CRNX/V3/NYA100NOR_S_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_EN.rnx.gz \
    ppp --cggtts -c tutorials/config/survey/cpp_kf.json
