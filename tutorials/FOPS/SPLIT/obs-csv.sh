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
# --csv: output to CSV directly after split
./target/release/rinex-cli \
    -P "GPS;L1C,C1C" \
    --fp $TEST_FILE \
    split $NOON \
    --csv

# --zip: gzip compress your CSV directly
./target/release/rinex-cli \
    -P "GPS;L1C,C1C" \
    --zip \
    --fp $TEST_FILE \
    split $NOON \
    --csv
