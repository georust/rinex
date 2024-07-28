#! /bin/sh
# Split 24h observation serie into a batch of equal duration
EPOCH="2020-06-25T12:00:00 GPST" # split @ noon
FILTER="GPS" # GPS only
FILE=test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz

# Split batch file, generate gzip compressed files
./target/release/rinex-cli \
    -P $FILTER \
    --fp $FILE \
    split "$EPOCH"

# Analyze one of the generated file
./target/release/rinex-cli \
    --fp WORKSPACE/ESBC00DNK_R_20201770000_01D_30S_MO/OUTPUT/ESBC00DNK00DNK_R_20201772359_01D_30S_MO.crx
