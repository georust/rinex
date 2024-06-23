#!/bin/sh
# QC report example when providing a complete/modern meteo file
DATA_DIR=test_resources

# This is our only example of Wind Direction observations
METEO=$DATA_DIR/MET/V2/abvi0010.15m

# Example: complete analysis
./target/release/rinex-cli -f $METEO qc
