#! /bin/sh
TEST_FILE=test_resources/MET/V3/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz

# --filegen: open context and synthesize what we can from it
# --csv: export to CSV
./target/release/rinex-cli \
    -q \
    --fp $TEST_FILE \
    filegen --csv

# --filegen: open context and synthesize what we can from it
# --zip: preserve GZIP compression
./target/release/rinex-cli \
    -q \
    --fp $TEST_FILE \
    --zip \
    --filegen --csv
