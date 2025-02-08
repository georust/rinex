#!/bin/sh
# Observation record decimation and synthesis

./target/release/rinex-cli \
    -P "decim:3 h" \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    filegen
