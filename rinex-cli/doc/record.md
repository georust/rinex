Observation RINEX
=================

When analyzing Observation RINEX, one plot per encountered observations
is generated.
Receiver clock offsets are plotted if they were identified.

Let's analyze Observations for 4 vehicles: 

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --retain-sv R01,R08,G21,G31
```

The received signal power analysis for example, extracted from the analysis report

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_ssi.png">

It is rapidly necessary to determine which vehicules can be encountered in a file.  
For this reason, we developped the `--sv-epoch` analysis, which helps determine which vehicule to focus on.

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --retain-sv G02,R02,R01,G01 \
    --sv-epoch
```

With the Sv per Epoch analysis, you know R01,R02 were both seen at 21:00UTC

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_sv_epoch.png">


When dealing with Observation RINEX, the following operations are most useful:

- `--sv-epoch`: vehicle(s) accross epochs identification
- `--observables`: observables identification
- `-w [DATETIME] [DATETIME]`: time windowing
- decimation: increase epoch interval - reduce data quantity
- `-R`, `-G`, `-C`, `-J`, `-E`, `-S`: quickly get rid of given GNSS constellation
- `--retain-constell`: focus on constellation(s) of interest
- `--retain-sv`: focus on vehicle(s) of interest 
- `--retain-obs`: focus on observation(s) of interest

Phase observations 
==================

Phase observations are "harder" to handle due to the carrier cycle ambiguities,
but also the most interesting data to the advanced user.

When phase observations are plotted, we always align them to the origin,
for easy phase variations comparison.

We also emphasize _possible_ cycle slips when plotting with a black symbol. 
For example L5 of G10 in `GRAS00FRA_R_2022` is affected:

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/gras00fra_g10phase.png">

4 micro (1 tick long) possible corruptions over this channel, which was used to sample L5 at high rate. 

Cycle slips may happen randomly, for a given channel and signal.   
To learn more about cycle slips, refer to the [processing section](processing.md).

Enhanced Observation analysis
=============================

Navigation data can be added on top of Observation RINEX provided with `--fp`.   

Ideally, Navigation data (ephemeris frames) were sampled with the same parameters, at the same time,
in the same environment. To proceed to enhanced analysis, specify the context with `--nav`.

The enhanced visualization depicts the Sv elevation angles accross encountered epochs,
along previous Observations. We do not apply interpolation for Observation / Navigation
frames to match in time, we simply exhibit shared epochs.

We can take advantage of the `--sv-epoch` opmode, which is designed in case of `--nav` enhanced
mode of operation, to depict epochs were both Ephemeris and Observations are shared.
In this example, we run it for all GPS vehicles:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --nav test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --retain-constell GPS --sv-epoch
```

From the resuling "sv.png" product:

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_gps_obs_nav_sv.png">

Triangles mark ephemeris frames (low rate) and circles mark observations (high rate).   
G25, G29, G31 and G12 in 20% to 35% portion of that day, have enough elevation angle information for the enhancement
to be interesting.

Let's plot it:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --nav test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --retain-sv G25,G29,G31,G12 \
    -w "2020-06-25 05:00:00 2020-06-25 10:00:00"
```
