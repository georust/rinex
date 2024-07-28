#!/bin/sh
# OBS_RINEX(Esbjerg) - OBS_RINEX(Mojn)
# This operation is very powerful to compare two GNSS receivers setup.
# Especially when two clocks are synchronized or is split in the setup.
# To perform the differentiation, you need a common timeframe and common signals.
# For the lack of a better example, we use MOJNDNK and ESBJRG (DNK) that provided
# data on the same day, and happen to be very close to one another
WORKSPACE=WORKSPACE
DATA_DIR=test_resources/CRNX/V3

# BDSBAS: S44
# EDNOS : S23,S26
#   Gal : >05
#   GPS : <20
FILTER="GPS,Gal;>E05;<G20"
TIMEFRAME=">2020-06-25T02:00:00 UTC" # skip 2hr (example)

# Generate ""differenced"" observation RINEX=obs(A)-obs(B)
./target/release/rinex-cli \
    -q \
    -P $FILTER "$TIMEFRAME" \
    --fp $DATA_DIR/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    diff $DATA_DIR/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz

# differential analysis
./target/release/rinex-cli \
    --fp $WORKSPACE/ESBC00DNK_R_20201770000_01D_30S_MO/DIFFERENCED.crx.gz
