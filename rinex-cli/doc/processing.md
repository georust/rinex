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

Example: in this file, only GPS L2 share a double phase code: "L2S" and "L2W".
With requesting the phase code analysis, we will differentiate "S" against "W"
for carrier 2. With the following command line, we focus on 3 vehicules.

As always, console or data visualization is possible, but usually we're only
interested in generating a plot:

```bash
rinex-cli --fp -f test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx \
    --phasediff --retain-sv G01,G07,G08 --plot 
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

## Cycle slips

Under development
