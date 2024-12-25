#/bin/sh

# PPP Surveying
# Glacier altitude is resolved by PPP nav from observations made by static ROVER
./target/release/rinex-cli \
    -z \
    --fp test_resources/OBS/V3/240506_glacier_station.obs.gz \
    --fp test_resources/NAV/V3/240506_glacier_station.nav.gz \
    --fp test_resources/SP3/COD0MGXFIN_20241270000_01D_05M_ORB.SP3.gz \
    ppp
