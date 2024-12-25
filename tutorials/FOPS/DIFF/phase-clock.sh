#!/bin/sh
# A clock is split into two separate receivers,
# sky is observed and we use identical GNSS RF signal/modulations
# to obtain the local clock behavior.
WORKSPACE=WORKSPACE
DATA_DIR=test_resources/OBS/V3

# Generate ""differenced"" observation RINEX=obs(A)-obs(B)
# -o: custom output name
#     we currently have no easy way to format dirty or incomplete
#     files like those
./target/release/rinex-cli \
    -o DIFFERENCED \
    --fp $DATA_DIR/OB713520.23O.gz \
    diff $DATA_DIR/gps.23O.gz

# Output "differenced" RINex as CSV directly.
# --gzip may apply here as well, to zip it directly
# -o: custom output name
#     we currently have no easy way to format dirty or incomplete
#     files like those
./target/release/rinex-cli \
    -o DIFFERENCED \
    --fp $DATA_DIR/OB713520.23O.gz \
    diff $DATA_DIR/gps.23O.gz \
    --csv

# Analyze any output product by loading it back into the toolbox
# ./target/release/rinex-cli \
#    --fp $WORKSPACE/OB713520/DIFFERENCED.23O.gz
