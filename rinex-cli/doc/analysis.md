Sv per epoch
============

Vehicles per epoch identification is requested with `--sv-epoch`.  
This mode will generate a plot that emphasize which vehicles
were encountered per epoch.

This analysis is very useful to emphasize data gaps in the Record,
per Satellite ID and GNSS constellation.

The analysis generates a file name `sv.png`:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.rnx \
    --retain-sv G01,G08,R04,R08,R09 --sv-epoch
```

<img align="center" width="650" src="https://github.com/georust/rinex/blob/main/doc/plots/sv_esbc00dnk.png">


In case Differential context is activated (`--nav`) determining
which vehicles were both sampled in
Ephemeris and Observation Context becomes rapidly mandatory.
Otherwise, we just don't know which vehicle could be a good candidate
for Differential operations.

`--sv-epoch` has a special behavior when `--nav` context is provided,
in this scenario, we exhibit vehicles encountered in both files
accross epochs, and this help decide which one to use later on.

This special behavior will work well if you select a unique vehicle,
otherwise plot becomes rapidly messy:

```bash
rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -P G01,G08,R04,R08,R09 --sv
```

<img align="center" width="650" src="https://github.com/georust/rinex/blob/main/doc/plots/sv_diff_esbc00dnk.png">

With this command, user can rapidly determine which vehicle is eligible for
RINEX differential processing. In this example, R04, R08 and R09 are excellent candidates,
because most of the Observation context is covered by Ephemeris.

To learn more about differential processing, refer to the 
Differential proceesing operations described
[in this page](https://github.com/georust/rinex/blob/main/rinex-cli/doc/processing.md).

Sample rate analysis
====================

Sample rate steadyness might be important in operations to perform.  
`--epoch-hist` performs a histogram analysis of all epoch durations across `-fp`.  

For example, `ESBC00DNK_R_20201` is a large file with steady 30s sample rate.

```bash
rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz --epoch-hist
```

<img align="center" width="650" src="https://github.com/georust/rinex/blob/main/doc/plots/esbc00dnk_hist.png">

When applying to non-steady files, this plot emphasizes the average (dominant) sample rate and the amount of anomalies.   
In this example, 16 epochs were generated, dominant sample rate is 30s and 2 epochs are missing.

<img align="center" width="650" src="https://github.com/georust/rinex/blob/main/doc/plots/hist2.png">
