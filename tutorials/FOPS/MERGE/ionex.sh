#! /bin/sh
DIR=test_resources/IONEX/V1

# merge
./target/release/rinex-cli \
    --fp $DIR/CKMG0080.09I.gz
    merge $DIR/CKMG0090.21I.gz

# analyze any output product, by loading it back into the toolbox
