#! /bin/sh
DIR=test_resources/IONEX/V1

# merge
./target/release/rinex-cli \
    --fp $DIR/CKMG0080.09I.gz
    merge $DIR/CKMG0090.21I.gz

# analyze
# ./target/release/rinex-cli \
#    --fp WORKSPACE/CKMG
