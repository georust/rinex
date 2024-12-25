#!/bin/sh

TESTPOOL=test_resources/MET/V3
FILEPATH=$TESTPOOL/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz

# Compressed Gzip observations 
# -P decim:30min: 50% record decimation
# --filegen: synthesize resulting RINEx
./target/release/rinex-cli \
    -P "decim:30 min" \
    --fp $FILEPATH \
    filegen

# Decimated record analysis (HTML geodetic report)
./target/release/rinex-cli \
    -q \
    -o decimated-half-hr \
    --fp WORKSPACE/POTS00DEU/POTS00DEU_R_20232540000_01D_MM.rnx

# CSV example: 
# -P decim:3min 50% record decimation
# --csv: export to CSV
# --zip: zip the CSV file directly
./target/release/rinex-cli \
    -P "decim:30 min" \
    --fp $FILEPATH \
    filegen \
    --csv
