#! /bin/sh
DIR=test_resources/CRNX/V3
FILTER="GPS" # GPS only

# Split batch file, generate gzip compressed files
./target/release/rinex-cli \
    -P $FILTER \
    --fp $DIR/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    merge $DIR/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz

# analyze merged file
./target/release/rinex-cli \
    --fp WORKSPACE/ESBC00DNK_R_20201770000_01D_30S_MO/OUTPUT/ESBC00DNK00DNK_R_20201772359_01D_30S_MO.crx
