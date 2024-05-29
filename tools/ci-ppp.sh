#! /bin/bash
# Small script to test PPP capabilities across toolbox updates

# make sure we're in the correct setup
cargo build --all-features -r

LOGS="logs/ci/ppp"
APP="./target/release/rinex-cli -q"

PPP_CONTEXT="-f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -f test_resources/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    -f test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz"
    
mkdir -p $LOGS
export RUST_LOG=trace

# Default PPP | GPS
$APP $PPP_CONTEXT -P GPS -p | tee $LOGS/gps_default.logs
# Default PPP + CGGTTS | GPS
$APP $PPP_CONTEXT -P GPS -p --cggtts | tee $LOGS/gps_default_cggtts.logs
# Default PPP | GAL
$APP $PPP_CONTEXT -P GAL -p | tee $LOGS/galileo_default.logs
# Advanced SPP | GPS
$APP $PPP_CONTEXT -P GPS -p -c config/survey/spp_lsq.json | tee $LOGS/gps_spp_advanced.logs
# # Advanced SPP | GAL
$APP $PPP_CONTEXT -P GPS -p -c config/survey/spp_lsq.json | tee $LOGS/galileo_spp_advanced.logs
