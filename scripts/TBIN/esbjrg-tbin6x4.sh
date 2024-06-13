#! /bin/sh
# Split 24h observation serie into a batch of 4 6h observation series.
# Use the Preprocessor to determine which constellation and SV you want to preserve
FILTER="GPS;>G08" # in this example, the batch will only contain GPS >08

FILE=test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz

./target/release/rinex-cli \
    -P $FILTER \
    -f $FILE tbin "6 hour"
