#!/bin/sh
# QC report example when providing a 24h IONEX modeling (standalone)
DATA_DIR=test_resources
IONEX=$DATA_DIR/IONEX/V1/CKMG0020.22I.gz

# Example: complete analysis
./target/release/rinex-cli --fp $IONEX
