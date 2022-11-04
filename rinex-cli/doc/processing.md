Processing
==========

Some RINEX processing operations can be performed with this tool.

All of them are RINEX dependent. That means,
most actions are highly dependent to which kind of RINEX
was provided with `-fp`. 
Advanced RINEX processing may require a Navigation file
to be also attached, this is performed with `--nav`.

## Phase Code differenciation

Phase Code analysis can be performed on Observation RINEX with `--phasediff`.  
This analysis is very useful to determine correlations
and biases between different codes.

This operation differentiates raw phase data
between two difference phase code sampled at the same carrier frequency.
Obviously this requires modern RINEX where multiple carrier and codes
were sampled.

Refer to page 11 of
[this ESA analysis](http://navigation-office.esa.int/attachments_12649498_1_Reichel_5thGalSciCol_2015.pdf)

It is highly recommended to focus on vehicules of interest 
when performing such analysis.

Example (1) `ACOR00ESP_R_20213550000_01D_30S_MO.rnx` contains enough information
to perform "L2S-L2W", meaning "S" code against "W" code on GPS carrier 2,
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


## Code differenciation

Code (Pseudo Range) analysis can be performed on Observation RINEX with `--codediff`
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

## Code Mutlipath Analysis

```bash
rinex-cli --fp -f test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --multipath --retain-sv G01,G07,G08 --plot 
```

## Cycle slips

Under development
