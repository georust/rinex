#!/bin/sh
# Real time surveying (BRDC) using observations from NYA100 (arctic).
# NYA100 & NYA200 are located on the same site.
#  Therefore, they share identical conditions, for example we can share NAV/V3 datasets.
DATA_DIR=test_resources

# Force new report (-f)
# Silent reporting (-q): open on last run only
# PPP solutions (ppp) with single Signal=C1C
./target/release/rinex-cli \
    -P "Gal;C1C" \
    -f -q -o "Gal-L1" \
    --fp $DATA_DIR/CRNX/V3/NYA200NOR_R_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_EN.rnx.gz \
    ppp -c tutorials/config/survey/spp_lsq.json

# Append PPP solutions (ppp) with single signal=C7Q
./target/release/rinex-cli \
    -P "Gal;C7Q" \
    -f -q -o "Gal-L5" \
    --fp $DATA_DIR/CRNX/V3/NYA200NOR_R_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_EN.rnx.gz \
    ppp -c tutorials/config/survey/spp_lsq.json

# Dual channel Navigation (C1C+C7Q) compare to past runs
./target/release/rinex-cli \
    -P "Gal;C1C,C7Q" \
    -f -q -o "Gal-L1L5" \
    --fp $DATA_DIR/CRNX/V3/NYA200NOR_R_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_EN.rnx.gz \
    ppp -c tutorials/config/survey/cpp_kf.json

# Generate CGGTTS solutions for past run as well.
# Open report.
./target/release/rinex-cli \
    -P "Gal;C1C,C7Q" \
    -o "Gal-L1L5" \
    --fp $DATA_DIR/CRNX/V3/NYA200NOR_R_20241240000_01D_30S_MO.crx.gz \
    --fp $DATA_DIR/NAV/V3/NYA100NOR_S_20241240000_01D_EN.rnx.gz \
    ppp --cggtts -c tutorials/config/survey/cpp_kf.json
