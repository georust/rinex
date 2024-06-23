#!/bin/sh
# QC report example when providing a complete/modern meteo file
DATA_DIR=test_resources

# This is our only example that contains Wind Direction + Wind speed observations.
# When wind direction observations are present, you get a compass view.
# When both are present, the wind direction compass is indexed on the speed observations.
METEO=$DATA_DIR/MET/V2/abvi0010.15m

# Example: complete analysis
./target/release/rinex-cli -f $METEO qc
