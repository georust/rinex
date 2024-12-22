#! /bin/bash
#############################
# RNX2CRX (v3 =modern files)
#############################
RNX2CRX=./target/release/rnx2crx
FOLDER=test_resources/OBS/V3

# V3 (modern) with V2 like filename:
# use --short to generate similar name
$RNX2CRX --short $FOLDER/pdel0010.21o

# Use --zip to compress at the same time
$RNX2CRX --short --zip $FOLDER/pdel0010.21o

# V3 example
# Use --zip to preserve Gzip compression
$RNX2CRX --zip $FOLDER/240506_glacier_station.obs.gz
