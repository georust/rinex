#! /bin/bash
#############################
# CRX2RNX (v1 =old revision)
#############################
CRX2RNX=./target/release/crx2rnx
FOLDER=test_resources/CRNX/V1

# since the toolbox prefers V3 format, we need to use
# --short to remain in the V1/V2 standard
$CRX2RNX --short $FOLDER/eijs0010.21d

# gzip compress directly
$CRX2RNX --short --zip $FOLDER/zegv0010.21d
