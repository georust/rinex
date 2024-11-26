#! /bin/bash
#############################
# CRX2RNX (v1 =old revision)
#############################
CRX2RNX=./target/release/crx2rnx
FOLDER=test_resources/CRNX/V1

# short file example
$CRX2RNX $FOLDER/ACOR00ESP_R_20213550000_01D_30S_MO.crx

# force .gz compression manually on the output
$CRX2RNX --gz $FOLDER/ACOR00ESP_R_20213550000_01D_30S_MO.crx

# realistic file, .gz compression is preserved
$CRX2RNX $FOLDER/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
