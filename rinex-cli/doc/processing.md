Processing
==========

This tool implements several RINEX processing algorithms.    
All of them are RINEX format dependent. That means,
one can only perform a desired operation if the provided file matches
the expected kind of RINEX.

Some RINEX processing algorithms require Navigation data (ephemeris),
to be passed, and we use the `--nav [FILE]` flag to do that,
while `--fp` remains how you give the base file.

This library is implemented such as Military codes are supported just like others.
They're just not tested due to obvious lack of data.

GNSS Signal Combination
=======================

This tool supports standard GNSS recombinations, especially for modern RINEX. 

Refer to this page for [thorough documentation](gnss-combination.md).  
It is important to understand GNSS signal recombinations and what they can represent.  

The RINEX tool suites can estimate the following code biases:

- the DCB code biases 
- the MP code biases

Differential Code Biases (DBCs)
===============================

DBC analysis is requested with `--dcb`.  
It is very similar to a GNSS recombination, so if you've already gone through the previous page,
you pretty much know how to run such an analysis.

This analysis is very useful to determine correlations and biases between different codes.  
The results are $K_i$ terms in the phase signal model from the previous page.

Refer to pages 11 and 12 of
[this ESA analysis](http://navigation-office.esa.int/attachments_12649498_1_Reichel_5thGalSciCol_2015.pdf)
for more information on DBCs.

When requesting this operation, you get a "dcb.png" plot.  
Like previous recombinations, one can stack the `--dcb` to other recombinations and also a record analysis. 
Because data is not modified in place, and both results are exposed in seperate plots.

`ESBC00DNK_R_2020` has enough information to evaluate

* 1C/1P for GLO L1 (1)
* 2C/2P for GLO L2 (2)
* 2L/2W for GPS L2 (3)

Analyze 1C/1P and 2C/2P with:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
        -P G07 --dcb
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_ph_dcbs.png">


Code Multipath biases
=====================

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


MP analysis is summoned with `--mp`.

In this example, `ESBC00DNK_R_2020` vehicle GPS#13 
has enough data to compute MP ratios:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
        -P G13 --dcb --mp
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_g13_dcb_mp.png">

Differential Processing
=======================

When moving to more advanced RINEX processing,
Navigation RINEX (Ephemeris) must be provided with `--nav`.

In this mode, `--fp` is expected to be an Observation RINEX.

Let's remind the user that in this mode, `--sv-epoch` helps
exhibit which vehicles share Ephemeris and Observations for a given epoch
or epoch range. This feature is very important do determine
which vehicle is a good candidate for the operations that follow.

## Cycle slips analysis

Under development
