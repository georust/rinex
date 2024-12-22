#! /bin/sh
TEST_FILE=test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz

# in this scenario, we want to extract L1 and L5 to CSV for both
# GPS and Galileo. We use two filters to generate two seperate files.
# We restrict to one hour time frame.

TIMEFRAME=">=2020-06-05T12:00:00 GPST;<2020-06-25T13:00:00 GPST"
GPS_FILTER="GPS;C1C,C5Q,D1C,D5Q"
Gal_FILTER="GAL;C1C,C5Q,D1C,D5Q"

# --filegen: open context and synthesize what we can from it
# --csv: export to CSV
./target/release/rinex-cli \
    -q -o "GPS-L1L5" \
    -P "$TIMEFRAME" "$GPS_FILTER" \
    --fp $TEST_FILE \
    filegen --csv

# --filegen: open context and synthesize what we can from it
# --csv: export to CSV
./target/release/rinex-cli \
    -q -o "Gal-E1E5" \
    -P "$TIMEFRAME" "$Gal_FILTER" \
    --fp $TEST_FILE \
    filegen --csv
