#! /bin/bash
#############################
# RNX2CRX (v1 =old revision)
#############################
RNX2CRX=./target/release/rnx2crx
FOLDER=test_resources/OBS/V2

# short file example
# Use --short to generate similar name
$RNX2CRX --short $FOLDER/AJAC3550.21O

# --zip compress directly
$RNX2CRX --short --zip $FOLDER/AJAC3550.21O
