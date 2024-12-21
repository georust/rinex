#!/bin/sh

# Compressed Gzip observations decimation, decompression and output
# --crx2rnx and --rnx2crx obviously only apply when synthesizing RINEx.
./target/release/rinex-cli \
    -P decim:3h \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --crx2rnx \
    filegen

# Compressed Gzip observations decimation, decompression and export to CSV.gz
./target/release/rinex-cli \
    -P decim:3h \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    filegen \
    --csv \
    --gzip
