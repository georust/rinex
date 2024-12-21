#!/bin/sh

# Compressed IONex maps
# -P decim:2h: =%50 record decimation
# --filegen: synthesize resulting IONex
# --zip: force gzip compression
./target/release/rinex-cli \
    -P decim:2h \
    --zip \
    --fp test_resources/IONEX/V1/CKMG0020.22I.gz \
    filegen

# Decimated record analysis (HTML geodetic report)
./target/release/rinex-cli \
    -o decimated-2h \
    --fp WORKSPACE/CKMG0020/CKMG0020.22I.gz

# CSV export example: 
# --zip: csv+gzip
./target/release/rinex-cli \
    -P decim:2h \
    --zip \
    --fp test_resources/IONEX/V1/CKMG0020.22I.gz \
    filegen \
    --csv
