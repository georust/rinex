#! /bin/sh
DIR=test_resources/OBS/V3

# example: preprocessing prior merge
FILTER="C1C,L1C,L2P,C2P"

# -P: preprocess
# --short: generate a similar V2-like name 
# merge: B into A
./target/release/rinex-cli \
    -P $FILTER \
    --short \
    --fp $DIR/VLNS0010.22O \
    merge $DIR/VLNS0630.22O

# Use --zip to automatically gzip your output products
./target/release/rinex-cli \
    -P $FILTER \
    --short \
    --zip \
    --fp $DIR/VLNS0010.22O \
    merge $DIR/VLNS0630.22O

# Load any synthesized product back into the toolbox, to analyze it
./target/release/rinex-cli \
    --fp WORKSPACE/VLNS0010/VLNS0010.22O.gz
