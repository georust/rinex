#! /bin/bash

# V1 examples
FOLDER=test_resources/CRNX/V1

# since the toolbox prefers V3 format, we need to use
# --short to remain in the V1/V2 standard
crx2rnx --short $FOLDER/eijs0010.21d

# gzip compress directly
crx2rnx --short --zip $FOLDER/zegv0010.21d
