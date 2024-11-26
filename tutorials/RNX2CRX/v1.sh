#! /bin/bash
#############################
# RNX2CRX (v1 =old revision)
#############################
RNX2CRX=./target/release/rnx2crx
FOLDER=test_resources/OBS/V2

# short file example
$RNX2CRX $FOLDER/AJAC3550.21O

# force .gz compression manually on the output
$RNX2CRX --gz $FOLDER/AJAC3550.21O

# realistic file
$RNX2CRX --gz $FOLDER/npaz3550.21o
