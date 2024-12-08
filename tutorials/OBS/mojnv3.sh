#! /bin/sh

# this is a modern V3 crinex that we directly parse.
# the report only contains its analysis (single file scenario).
# Multi GNSS RINEX being quite lengthy, it is recommended to shrink
# to the type of data you're interested in (to render faster).

# -f : force report synthesis (to always generate =single file scenario)
# -P : retain a few datasets
./target/release/rinex-cli \
    -f \
    -P "G08,G15,G31,C15,C31,R03,E11" \
    --fp test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz

