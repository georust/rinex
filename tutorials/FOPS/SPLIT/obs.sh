#! /bin/sh

NOON="2020-06-25T12:00:00"
TEST_FILE=test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz 

# Although the --short(-s) flag may apply here, to form V2 like (shortened)
# file names; it is quite convenient to work with longer (V3+) file names
# when dealing a batch of RINex files, because the identity of each
# individual file within the batch can be represented.
#
# In this example, we create a properly named batch, because we're working
# with standardized files.

# -P: any preprocessing pipeline may apply
./target/release/rinex-cli \
    -P "GPS;L1C,C1C" \
    --fp $TEST_FILE \
    split $NOON

# --zip: force gzip compression on your output product
./target/release/rinex-cli \
    -P "GPS;L1C,C1C" \
    --zip \
    --fp $TEST_FILE \
    split $NOON

# --crx2rnx: decompress the CRINex format and output as RINex instead
./target/release/rinex-cli \
    -P "GPS;L1C,C1C" \
    --crx2rnx \
    --zip \
    --fp $TEST_FILE \
    split $NOON

# Analyze any output product by loading it back in to the toolbox.
# Here we analyze the morning file set
./target/release/rinex-cli \
    --fp WORKSPACE/ESBC00DNK/ESBC00DNK_R_20201770000_01D_30S_MO.crx
