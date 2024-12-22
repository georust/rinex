#! /bin/sh
DIR=test_resources/MET/V2

# Use --short to generate similar V2-like files
./target/release/rinex-cli \
    --short \
    --fp $DIR/clar0020.00m \
    merge $DIR/abvi0010.15m

# Add --zip to gzip compress at the same time
./target/release/rinex-cli \
    --zip \
    --short \
    --fp $DIR/clar0020.00m \
    merge $DIR/abvi0010.15m

# analyze any output product by loading it back into the toolbox
./target/release/rinex-cli \
    --fp WORKSPACE/clar0020/CLAR0020.00M.gz
