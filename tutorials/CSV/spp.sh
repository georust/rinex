#! /bin/sh
NAV=test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz
OBS=test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz

# Same example as obs.sh but we add NAV RINEX as well,
# which gives SPP compliancy. In this mode,
# we can export both Observations and Orbital states.

# Since 1C and 5Q represents Both L1/E2 and L1/E5 depending
# of which constellation we're talking about,
# we need to proceed in two steps, first GPS then Galileo.
FILTER="GPS;C1C,C5Q,D1C,D5Q;>=2020-06-05T12:00:00 GPST;<2020-06-25T13:00:00 GPST"

# -q: since we're generating data, we're not interested in opening the workspace
# --csv: export to CSV
./target/release/rinex-cli \
    -q -o "GPS-L1L5" \
    -P "$FILTER" \
    --fp $OBS \
    --fp $NAV \
    filegen --csv

FILTER="Gal;C1C,C5Q,D1C,D5Q;>=2020-06-05T12:00:00 GPST;<2020-06-25T13:00:00 GPST"

# -q: since we're generating data, we're not interested in opening the workspace
# --csv: export to CSV
./target/release/rinex-cli \
    -q -o "Gal-E1E5" \
    -P "$FILTER" \
    --fp $OBS \
    --fp $NAV \
    filegen --csv
