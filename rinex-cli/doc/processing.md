Processing
==========

Some RINEX processing operations can be performed with this tool.

All of them are RINEX dependent. That means,
possible actions highly depend on which type of RINEX was provided with `--fp`.  
Advanced RINEX processing may require a Navigation file
to also be attached with `--nav`.

## Phase Code differential analysis

Phase Code analysis can be performed on Observation RINEX with `--phasediff`.  
This analysis is very useful to determine correlations
and biases between different codes.

This operation differentiates phase data
between two phase codes sampled against the same carrier frequency.

Refer to page 11 of
[this ESA analysis](http://navigation-office.esa.int/attachments_12649498_1_Reichel_5thGalSciCol_2015.pdf)

It is highly recommended to focus on vehicules of interest 
when performing such analysis.

Example (1) `ACOR00ESP_R_20213550000_01D_30S_MO.rnx` contains enough GPS data 
to perform "L2S-L2W", meaning "S" code against "W" code on carrier 2.
and also "L2C-L2P" meaning "C" against "P" code on Glonass carrier 2.

For information, when performing "x" against "y", the reference is select by alphabetical order.
But that is not very important, as we're only interested in relative variations, when
performing such analysis.

```bash
rinex-cli --fp test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --plot \
    --phasediff \
    --retain-sv G01,G07 # will restrict to L2S-L2W
```

Now let's analyze both L2S/L2W (2 vehicules) and L2C/L2P (3 vehicules),
as we have the hability to perform
all possible differentiation against consistent carriers:

```bash
rinex-cli --fp test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --plot \
    --phasediff \
    --retain-sv G01,G07,R18,R19,R20 
```

Example(2) `ESBC00DNK_R_20201770000_01D_30S_MO` has enough information
to study "L2L/L2W" for GPS2, "L1P/L1C", "L2C/L2P" for Glonass:

```bash
rinex-cli --fp test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --plot \
    --phasediff \
    --retain-sv G02,G05,G07,R01,R02,R11,R12
```

## Code (Pseudo Range) differential analysis

Code (PR) analysis can be performed on Observation RINEX with `--codediff`
in similar fashion.
This analysis exhibits offset and drift tendencies between different codes.

Refer to page 12 of
[the ESA analysis](http://navigation-office.esa.int/attachments_12649498_1_Reichel_5thGalSciCol_2015.pdf)

It is highly recommended to focus on vehicules of interest 
when performing such analysis.

```bash
rinex-cli --fp -f test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --codediff --retain-sv G01,G07,G08 --plot 
```

## NAV / OBS Shared epochs

Before we move on to more advanced processing that involve
combining Observation to Navigation RINEX,
let's remind that `--sv-epoch` in case of differential analysis (`--nav`)
has the ability to exhibit shared epochs between Observations and Ephemeris.  
This is useful to determine which vehicule to focus on in the following operations.

## Code Multipath (MP) analysis

Another type of differential analysis is the Code Multipath (MP) analysis.

MP ratios (so called "CMC" for Code Minus Carrier metrics) 
are formed by combining Phase and PR observations sampled for different carrier frequencies.

The MP results are presented against the elevation angle of the satellite(s) of interest.  
Thefore, MP analysis requires both an Observation RINEX (`--fp`) and Navigation context (`--nav`)
to be passed.

Usually the analysis is performed against a single vehicule, but this library
allows the results to be displayed for several vehicules at once.
There is no reason Military codes would not work just like others with this library.

For example, `ESBC00DNK_R_2020` has enough data for GPS vehicule 13,
to compute MP for codes "1C", "2W" and "5Q".
We also provide the whole context for demonstration purposes (with ideal sampling scenario):

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --nav test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --multipath \
    --retain-sv G07 \
    --plot 
```

## Cycle slips analysis

Under development
