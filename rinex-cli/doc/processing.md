Processing
==========

This tool implements several RINEX processing algorithms.  
All of them are RINEX format dependent. That means,
one can only perform a desired operation if the provided file matches
the expected kind of RINEX.

Some Differential RINEX processing algorithms are also supported.
In this scenario, we expect Observation RINEX to be passed with `--fp` 
and Navigation context to be passed with `--nav`.

This library is implemented such as Military codes are supported just like others.
They're just not tested due to obvious lack of data.

Phase Differential Code Biases (DBCs)
=====================================

Phase DBCs can be evaluated on Observation RINEX with `--phase-dcb`.  
This analysis is very useful to determine correlations and biases between different codes.  

This operation substracts a reference phase point to another phase observation,
as long as they were sampled against the same carrier frequency.

Refer to page 11 of
[this ESA analysis](http://navigation-office.esa.int/attachments_12649498_1_Reichel_5thGalSciCol_2015.pdf)

When requesting this operation, you get a "phase-dcb.png" plot.  
It is highly recommended to focus on vehicules of interest when performing such analysis.  

Example: `ESBC00DNK_R_2020` has enough information to evaluate

* 1C/1P for GLO L1 (1)
* 2C/2P for GLO L2 (2)
* 2L/2W for GPS L2 (3)

Get (1)+(2) for R09 and R18 with:

```bash
rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --phase-dcb \
    --retain-sv R09,R18
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_glo_ph.png">

Three very linear phases took place during that day, with huge data gaps in between
(channel stopped or vehicule out of sight).   
Focusing on one of these phases helps determine how linear they were.   
To do so, we can use set a time window with `-w`:

```bash
rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -w "2020-06-25 10:00:00 2020-06-25 18:00:00" \
    --phase-dcb \
    --retain-sv R09,R18 
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_glo_ph_zoom.png">

Now let's introduce a GPS vehicule so we also get (3):

```bash
rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -w "2020-06-25 09:00:00 2020-06-25 12:00:00" \
    --phase-dcb \
    --retain-sv R09,R18,G31
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_glogps_ph_zoom.png">

Pseudo Range Differential Code Biases (DBCs) estimates
======================================================

In similar fashion, Pseudo Range DBCs can be estimated with `--pr-dcb`.   
It it the exact same approach, but applied to Pseudo Range observations.

Refer to page 12 of
[the ESA analysis](http://navigation-office.esa.int/attachments_12649498_1_Reichel_5thGalSciCol_2015.pdf)

Example: `ESBC00DNK_R_2020` can evaluate

* 1C/1W for GPS(L1) (1)
* 2L/2W for GPS(L2) (2)
* 1P/1C for GLO(L1) (3)
* 2C/2P for GLO(L2) (4)

Get (1)+(2) with G06 and G31:

```bash
rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --retain-sv G06,G31 \
    --pr-dcb \
    -w "2020-06-25 18:00:00 2020-06-25 20:30:00" 
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_gps_prdiff.png">

Focus on observations you're interested in, to get the DBCs you're interested in.  
Get only (1) for G06 and G31:

```bash
rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --retain-obs C1C,C1W \  
    --retain-sv G06,G31 \
    --pr-dcb \
    -w "2020-06-25 18:00:00 2020-06-25 23:00:00"
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_gps_pr1c1w.png">

PR + PH DCBs
============

Both previous analysis can be performed at once, by requesting
`--phase-dcb` and `--pr-dcb` at the same time.


Differential Processing
=======================

When moving to more advanced RINEX processing,
Navigation RINEX (Ephemeris) must be provided with `--nav`.

In this mode, `--fp` is expected to be an Observation RINEX.

Let's remind the user that in this mode, `--sv-epoch` helps
exhibit which vehicules share Ephemeris and Observations for a given epoch
or epoch range. This feature is very important do determine
which vehicule is a good candidate for the operations that follow.

## Code Multipath (MP) analysis

MP ratios (so called "CMC" for Code Minus Carrier metrics) 
are formed by combining Phase and PR observations sampled against different carrier frequencies
and taking the frequency ratios into account (normalization).

The MP results are presented against the elevation angle of the satellite(s) of interest,
because in this study we're interested in determining which elevation angle gave
fewer multipath negative effects. This analysis can be used to evaluate
different antennas against multipath effects.

A very compelling use case of MP code analysis
can be
[found here](https://www.taoglas.com/wp-content/uploads/pdf/Multipath-Analysis-Using-Code-Minus-Carrier-Technique-in-GNSS-Antennas-_WhitePaper_VP__Final-1.pdf).

In this example, `ESBC00DNK_R_2020` vehicule GPS#13 
has enough data to compute MP ratio for codes "1C", "2W" and "5Q".
This operation requires combining the associated Ephemeris, that we also provide
for demonstration purposes:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --nav test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --multipath \
    --retain-sv G13
```

## Cycle slips analysis

Under development
