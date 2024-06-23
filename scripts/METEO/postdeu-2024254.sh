#!/bin/sh
# QC report example when providing a complete/modern meteo file
DATA_DIR=test_resources

# Report automatically adapts to provided context
METEO=$DATA_DIR/MET/V3/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz

# Example:
#  Skip first five minutes of that day (example)
FILTER=">2023-09-11T00:05:00 UTC"

./target/release/rinex-cli -P $FILTER -f $METEO qc
