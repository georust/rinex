#! /bin/sh
set -e
##############################################
# binaries tester: to be used in CI and
# provide at least basic means to test our CLI
##############################################
cargo build --all-features -r

#################
# CRX2RNX (V1)
#################
./target/release/crx2rnx \
    -f test_resources/CRNX/V1/delf0010.21d

echo "CRN2RNX (V1) OK"

#################
# CRX2RNX (V3)
#################
./target/release/crx2rnx \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz

echo "CRN2RNX (V3) OK"

#################
# RNX2CRX (V2)
#################
./target/release/rnx2crx \
    -f test_resources/OBS/V2/delf0010.21o

echo "RNX2CRX (V2) OK"

#################
# RNX2CRX (V3)
#################
./target/release/rnx2crx \
    -f test_resources/OBS/V3/pdel0010.21o

echo "RNX2CRX (V3) OK"

#############################
# 3. OBS RINEX identification
#############################
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -i -a

#############################
# 4. NAV RINEX identification
#############################
./target/release/rinex-cli \
    -f test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -i -a

####################
# 5. OBS RINEX merge
####################
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -m test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz

####################
# 6. OBS RINEX split
####################
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -s "2020-06-25T10:00:00 UTC"

###########################
# 7. OBS RINEX time binning
###########################
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --tbin "4 hour"

######################
# 8. GNSS combinations
######################
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -g --if --nl --wl --mw --mp --dcb --gf

#################
# 9. OBS RINEX QC
#################
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -Q

####################################
# 9. (advanced): state of the art QC
####################################
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -f test_resources/SP3/GRG0MGXFIN_20201760000_01D_15M_ORB.SP3.gz \
    -f test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
    -Q

# ####################
# # 10. (advanced) SPP
# ####################
# ./target/release/rinex-cli \
#     -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
#     -f test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
#     -f test_resources/SP3/GRG0MGXFIN_20201760000_01D_15M_ORB.SP3.gz \
#     -f test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
#     -p --spp
# 
# ###############################
# # 11. (advanced) SPP RNX2CGGTTS
# ###############################
# ./target/release/rinex-cli \
#     -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
#     -f test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
#     -f test_resources/SP3/GRG0MGXFIN_20201760000_01D_15M_ORB.SP3.gz \
#     -f test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz \
#     -p --spp --cggtts
