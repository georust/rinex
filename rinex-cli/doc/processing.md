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

## Phase (PH) Differential Code analysis

Phase Code differential analysis can be performed on Observation RINEX with `--phase-diff`.  
This analysis is very useful to determine correlations and biases between different codes.

This operation differentiates phase data
between two phase codes sampled against the same carrier frequency.

Refer to page 11 of
[this ESA analysis](http://navigation-office.esa.int/attachments_12649498_1_Reichel_5thGalSciCol_2015.pdf)

It is highly recommended to focus on vehicules of interest when performing such analysis.  

The current interface is not powerful enough to display correctly the same Code analysis
against two different vehicules (work in progress), so proceed as described down below. 

Example (1) `ACOR00ESP_R_202` contains enough data to evaluate

* 2S/2W: "S" code against "W" code for GPS L2
* 2P/2C: "P" code against "C" code for GLO L2 

With the following command, we're left
with 2S/2W analysis, since we focused on a single GPS vehicule:

```bash
rinex-cli --fp test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --phase-diff \
    --retain-sv G01
```

By focusing on a Glonass vehicule, we get the 2P/2C analysis

```bash
rinex-cli --fp test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --phase-diff \
    --retain-sv R04
```

By focusing on two vehicules, we get both analysis at the same time

```bash
rinex-cli --fp test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --phase-diff \
    --retain-sv G01,R04
```

Example(2) `ESBC00DNK_R_2020` has enough information to evaluate

* 1C/1P for GLO L1 (1)
* 2C/2P for GLO L2 (2)
* 2L/2W for GPS L2 (3)

Get (1)+(2) with:

```bash
rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --phase-diff \
    --retain-sv R09 
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_glo_ph.png">

Three very linear phases took place during that day, we huge data gaps in between
(channel stopped or vehicule out of sight). To determine how linear those phases
really were, we set a time window to zoom in on one of them:

```bash
rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -w "2020-06-25 10:00:00 2020-06-25 18:00:00" \
    --phase-diff \
    --retain-sv R09 
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_glo_ph_zoom.png">

Very good linearity during midday and most of the afternoon.

Zoom in on (1)+(2)+(3) at the same time:

```bash
rinex-cli --fp test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    -w "2020-06-25 10:00:00 2020-06-25 18:00:00" \
    --phase-diff \
    --retain-sv R09,G01 
```

## Pseudo Range (PR) Differential Code analysis

PR Differential analysis is the same differential approach,
but we focus on Pseudo Range observations instead.

Refer to page 12 of
[the ESA analysis](http://navigation-office.esa.int/attachments_12649498_1_Reichel_5thGalSciCol_2015.pdf)

For example (1) `ACOR00ESP_R_2021` can evaluate

* 2S/2W for GPS(L2) (1)
* 2C/2P for GLO(L2) (2)

Process (1)

```bash
rinex-cli --fp test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --retain-sv G01 \
    --pr-diff
```

Analyze (1)+(2)

```bash
rinex-cli --fp test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --retain-sv G01,R21 \
    --pr-diff
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/acor00esp_pr.png">

Another example: `ESBC00DNK_R_2020` can evaluate:

* 1C/1W for GPS(L1) (1)
* 1P/1C for GLO(L1) (2)
* 2L/2W for GPS(L2) (3)
* 2C/2P for GLO(L2) (4)

(1)+(2)+(3)+(4) with:

```bash
rinex-cli --fp test_resources/CRNX/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --retain-sv G01,R21 \
    --pr-diff
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_prdiff.png">

PR codes have less data gaps than PH codes (previous analysis).  
Lets focus (1)+(2)+(3)+(4) on the end of the day like we did before

```bash
rinex-cli --fp test_resources/CRNX/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --retain-sv G01,R21 \
    -w "2020-06-25 10:00:00 2020-06-25 23:30:00" \
    --pr-diff
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_prdiff_zoom.png">

## PR + PH Differential Code analysis

A special `--code-diff` opmode can be passed to perform
both previous analysis in a more efficient implementation 
(less iterations, quicker implementation).

The following command produces the same results as demonstrated
in the two previous paragraphs

```bash
rinex-cli --fp test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --retain-sv R08 \
    --code-diff
```

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
