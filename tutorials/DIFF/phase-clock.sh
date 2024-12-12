#!/bin/sh
# A clock is split into two separate receivers,
# sky is observed and we use identical GNSS RF signal/modulations
# to obtain the local clock behavior.
WORKSPACE=WORKSPACE
DATA_DIR=test_resources/OBS/V3

# Generate ""differenced"" observation RINEX=obs(A)-obs(B)
# diff is a file operations: a RINEX is dumped, no report synthesized.
./target/release/rinex-cli \
    --fp $DATA_DIR/OB713520.23O.gz \
    diff $DATA_DIR/gps.23O.gz

# Open previous results: generate a report
./target/release/rinex-cli \
    --fp $WORKSPACE/OB713520/DIFFERENCED.23O.gz
