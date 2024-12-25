#! /bin/sh
DIR=test_resources/OBS/V3

# example: preprocessing prior merge
FILTER="C1C,L1C,L2P,C2P"

# -P: preprocess
# --short: generate a similar V2-like name 
# merge: B into A
# --csv: dump as CSV instead of RINex
./target/release/rinex-cli \
    -P $FILTER \
    --short \
    --fp $DIR/VLNS0010.22O \
    merge $DIR/VLNS0630.22O \
    --csv

# Use --zip to gzip your CSV product
./target/release/rinex-cli \
    -P $FILTER \
    --short \
    --zip \
    --fp $DIR/VLNS0010.22O \
    merge $DIR/VLNS0630.22O \
    --csv
