#! /bin/sh
# RINEX-Cli supports most GNSS file formats.
# In this example, we split one SP3 file into 8 3hr long SP3 files.
# We can use the preprocessor to modify output content at the same time,
# in this example, we only retain Glonass >15
FILTER=">R15"

FILE=test_resources/SP3/ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz

./target/release/rinex-cli \
    -P $FILTER \
    -f $FILE tbin "3 hour"
