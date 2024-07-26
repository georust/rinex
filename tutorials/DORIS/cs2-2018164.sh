#!/bin/sh
# QC report with standalone standard DORIS file
DATA_DIR=test_resources
# RINEX-Cli supports a single DORIS satellite per analysis.
DORIS=$DATA_DIR/DOR/V3/cs2rx18164.gz

# Example: complete analysis
./target/release/rinex-cli -f $DORIS qc
