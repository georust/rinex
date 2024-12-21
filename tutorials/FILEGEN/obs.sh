#!/bin/sh

TESTPOOL=test_resources/CRNX/V3
FILEPATH=$TESTPOOL/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz

TESTPOOL=test_resources/OBS/V3
FILEPATH=$TESTPOOL/DUTH0630.22O

# Compressed Gzip observations 
# -P decim:3h: record decimation
# --crx2rnx: seamless decompression
# --filegen: synthesize resulting RINEx
# NB: 
#   --crx2rnx and --rnx2crx obviously only apply when synthesizing RINEx.
./target/release/rinex-cli \
    -P decim:3h \
    --crx2rnx \
    --fp $FILEPATH \
    filegen

# Decimated record analysis (HTML geodetic report)
./target/release/rinex-cli \
    -o decimated-3h \
    --fp WORKSPACE/ESBC00DNK/ESBC00DNK_R_20201770000_01D_30S_MO.rnx

# CSV example: 
# -P decim:1h record decimation
# --csv: CRINEX decompression + export to CSV
# --zip: zip the CSV file directly
./target/release/rinex-cli \
    -P decim:3h \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    filegen \
    --csv \
