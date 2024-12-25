#/bin/sh
# RTK Surveying
# Glacier altitude is resolved by RTK nav from observations made by static ROVER
# with respect to remote static STATION NYA NOR Site#100
./target/release/rinex-cli \
    -P GPS \
    --fp DATA/JMF/2024/128/240507_glacierH18_rover.obs \
    rtk \
    --fp DATA/JMF/2024/128/NYA100NOR_S_20241280000_01D_30S_MO.crx.gz

# ./target/release/rinex-cli \
#    --fp DATA/JMF/2024/128/240507_glacierH184_rover.nav \
#    --fp DATA/JMF/2024/128/240507_glacierH184_rover.obs \
#    --fp DATA/JMF/2024/128/ESA0OPSFIN_20241270000_01D_05M_ORB.SP3.gz \
#    ppp | tee logs/test.tx

# Mode Labo Ultra PPP (Gal) fonctionne
# /target/release/rinex-cli -P Gal \
#    --fp DATA/JMF/2024/128/NYA100NOR_S_20241270000_01D_EN.rnx.gz \
#    --fp DATA/JMF/2024/128/NYA100NOR_S_20241270000_01D_30S_MO.crx.gz \
#    --fp DATA/JMF/2024/128/ESA0OPSFIN_20241270000_01D_05M_ORB.SP3.gz \
#    --fp DATA/JMF/2024/128/ESA0OPSFIN_20241270000_01D_30S_CLK.CLK.gz \
#    ppp
