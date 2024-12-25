#! /bin/sh

# Split a 24h time frame into 4x6hour
# When working with RINex batch, it is convenient to work with
# V3 (lengthy) file names like here, because they allow differentiating
# each individual file in the fileset. If you prefer working with V2,
# you can use --short.
#
# Any file operations may apply, that includes --crx2rnx, --rnx2crx
# and --zip for seamless compressions.

FILTER="GPS;>G08" # in this example, the batch will only contain GPS >08

TEST_FILE=test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz

# -P: any preprocessing pipeline may apply
./target/release/rinex-cli \
    -P $FILTER \
    --fp $TEST_FILE tbin "6 hour"

# Analyze any output product by loading it back into the toolbox
# ./target/release/rinex-cli \
#    --fp WORKSPACE/
