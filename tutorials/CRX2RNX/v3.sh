#! /bin/bash
#############################
# CRX2RNX (v3 =modern files)
#############################
CRX2RNX=./target/release/crx2rnx
FOLDER=test_resources/CRNX/V3

# V3 example
$CRX2RNX $FOLDER/ACOR00ESP_R_20213550000_01D_30S_MO.crx

# V3 example
# Use --short to generate a V2 file name
$CRX2RNX --short $FOLDER/ACOR00ESP_R_20213550000_01D_30S_MO.crx

# V3 example:
# use --zip to gzip compress at the same time
$CRX2RNX --short --zip $FOLDER/ACOR00ESP_R_20213550000_01D_30S_MO.crx

# V3 standard example
# use --zip to preserve gzip compression
$CRX2RNX $FOLDER/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
