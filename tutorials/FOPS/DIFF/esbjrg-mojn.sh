#!/bin/sh

# In this example, we run `diff` between two remote stations
# separated by a few hundreds meters. The clocks are remotely asynchronous.
# Only the same physics and signals are differentiated and obtained as a result
DATA_DIR=test_resources/CRNX/V3

# -P example:
# BDSBAS: S44
# EDNOS : S23,S26
#   Gal : >05
#   GPS : <20
FILTER="GPS,Gal;>E05;<G20"
TIMEFRAME=">2020-06-25T02:00:00 UTC" # skip 2hr (example)

# First, we run `diff` and obtain the ""differenced"" observations (A-B)
rinex-cli \
    -q \
    -P $FILTER "$TIMEFRAME" \
    --fp $DATA_DIR/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    diff $DATA_DIR/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz

# Since the output product is still a RINEX. It may serve as a valid input product.
# We use it to generate a Qc analysis.
rinex-cli \
    --fp $WORKSPACE/ESBC00DNK_R_20201770000_01D_30S_MO/DIFFERENCED.crx.gz
