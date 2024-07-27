#! /bin/sh
DIR=test_resources/CLK/V3
FILTER="GPS" # GPS embedded clocks only

# merge
./target/release/rinex-cli \
    -P $FILTER \
    --fp $DIR/
    merge $DIR/

# analyze
./target/release/rinex-cli \
    --fp WORKSPACE/
