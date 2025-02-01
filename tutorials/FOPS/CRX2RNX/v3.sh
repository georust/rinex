#! /bin/bash
FOLDER=test_resources/CRNX/V3

# V3 example
crx2rnx $FOLDER/ACOR00ESP_R_20213550000_01D_30S_MO.crx

# V3 example
# Use --short to generate a V2 file name
crx2rnx --short $FOLDER/ACOR00ESP_R_20213550000_01D_30S_MO.crx

# V3 example:
# use --zip to gzip compress at the same time
crx2rnx --short --zip $FOLDER/ACOR00ESP_R_20213550000_01D_30S_MO.crx

# V3 standard example
# use --zip to preserve gzip compression
crx2rnx $FOLDER/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
