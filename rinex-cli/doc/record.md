Record analysis
===============

RINEX files are huge, complex and vary a lot.   
With these tools, we aim at providing an easy to use and efficient interface
to manipulate and visualize RINEX record.

Observation RINEX
=================

When analyzing Observation RINEX, one plot per kind of observations
is to be generated:

- "phase.png": Phase data points [n.a]
- "pseudorange.png": Pseudo Range data [pseudo distance]
- "ssi.png": Signal Strengths [dB]
- "doppler.png": Doppler shifts

An optionnal "clock-offset.png" will be generated, in case this RINEX
came with such information.

It is rapidly necessary to determine which vehicules can be encountered in the file. 
For this reason, we developped the `--sv-epoch` analysis, which helps determine which vehicule to focus on.

Locate GPS vehicules in `ESBC00DNK_R_2020`:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --retain-constell GPS \
    --sv-epoch
```

This generates "sv.png".

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_gps_sv.png">

For a file containing many vehicules per constellation like this one, 
we recommend focusing on a single one like we just did.

We'll focus on the first 3 hours of this file and we already know
we'll encounter G21, G27, G07, G09, G18 for instance. 

Analyze all observations for these vehicules:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --retain-sv G21,G09,G27 \
    -w "2020-06-25 00:00:00 2020-06-25 03:00:00"
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_gpsdoppler.png">

Doppler shifts were measured, they're exposed in "doppler.png".

When dealing with Observation RINEX, the following operations are most useful:

- `--retain-constell`: focus on constellations of interest
- `--retain-sv`: focus on vehicules of interest 
- `--sv-epoch`: plot encountered vehicules accros epochs
- `-w [DATETIME] [DATETIME]` zoom in on a slice of that day
- `--observables`: enumerate encountered observables per constellation.
- `--retain-obs`: focus on observation (codes) of interest.
This is one way to focus on a carrier signal of interest
- decimation: increse epoch interval / reduce data quantity

Phase data
==========

Phase observations are aligned at the origin for easy comparison and processing.  
The tool also emphasizes _possible_ cycle slips when plotting Phase observations,
with a black symbol for all epochs where the receiver declared a temporary loss of lock.  
For example L5 of G10 in `GRAS00FRA_R_2022` is affected:

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/gras00fra_g10phase.png">

4 micro (1sec) possible corruptions over this channel, which was used to sample L5 at high rate. 

Cycle slips may happen randomly, for a given channel and signal.   
To learn more about cycle slips, refer to the [processing section](processing.md).

Enhanced Observation analysis
=============================

Navigation context (Ephemeris) can be added on top of Observation RINEX
provided with `--fp`.   
Navigation context in this case is expect to have been sampled in identical conditions.  
This currently enhance the previous visualizations with satellite elevation angles.

Let's enhance the previous `ESBC00DNK_2020` visualization

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --nav test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --retain-sv G21,G09,G27 \
    -w "2020-06-25 00:00:00 2020-06-25 03:00:00"
    --plot
```

Plots now exhibit the elevation angle for vehicules we're interested in.
