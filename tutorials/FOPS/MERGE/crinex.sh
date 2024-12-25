#! /bin/sh
DIR=test_resources/CRNX/V3
FILTER="GPS;L1C" # GPS,L1 only

# Merge two files into a new file
# -P: apply preprocessing ops
# --short (-s): prefer shorter file names instead of V3/lengthy names
# --zip: preserve gzip compression, by generating a compressed output product
./target/release/rinex-cli \
    -P $FILTER \
    --short \
    --fp $DIR/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    merge $DIR/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz

# analyze any output product, by loading it back into the toolbox
./target/release/rinex-cli \
    --fp WORKSPACE/ESBC00DNK/ESBC1770.20D
