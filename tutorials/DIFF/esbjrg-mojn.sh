#!/bin/sh
# RINEX(A) - RINEX(B) observation
# This operation is typicall very powerful to compare two GNSS receivers
# to another, as long as the experimental setup is meaningful (for example: sync'ed or shared clocks).
# To perform the differentiation, we only need a common timeframe and common signals.
# For the lack of a better example, we use MOJNDNK and ESBJRG (DNK) that provided
# data on the same day, and happen to be very close to one another
WORKSPACE=WORKPACE
DATA_DIR=test_resources/CRNX/V3

FILTER=">E05;<E20" # Focus on Gal>05,<20 (any signals)
TIMEFRAME=">2020-06-25T02:00:00 UTC" # skip 2hr (example)

# Generate ""differenced"" observation RINEX=obs(A)-obs(B)
./target/release/rinex-cli \
    -q \
    -P $FILTER "$TIMEFRAME" \
    --fp $DATA_DIR/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    diff $DATA_DIR/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz

# Analyze differenced observations
./target/release/rinex-cli \
    --fp $WORKSPACE/ESBC00DNK_R_20201770000_01D_30S_MO/DIFFERENCED.crx.gz
