#! /bin/bash
#############################
# RNX2CRX (v3 =modern files)
#############################
RNX2CRX=./target/release/rnx2crx
FOLDER=test_resources/OBS/V3

# short file example
$RNX2CRX $FOLDER/pdel0010.21o

# force .gz compression manually on the output
$RNX2CRX --gz $FOLDER/pdel0010.21o

# realistic file, .gz compression is preserved
$RNX2CRX $FOLDER/240506_glacier_station.obs.gz
