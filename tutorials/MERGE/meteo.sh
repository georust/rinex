#! /bin/sh
DIR=test_resources/MET/V2

# merge
./target/release/rinex-cli \
    --fp $DIR/cari0010.07m \
    merge $DIR/clar0020.00m

# analyze
./target/release/rinex-cli \
    --fp WORKSPACE/
