RINEX analysis
==============

## Sv per epoch

Vehicules per epoch identification is requested with `--sv-epoch`.  
This mode will generate a plot that emphasize which vehicules
were encountered per epoch.

This analysis is very useful to emphasize data gaps in the Record,
per Satellite ID and GNSS constellation.

The analysis generates a file name `sv.png`:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.rnx \
    --retain-sv G01,G08,R04,R08,R09 --sv-epoch
```

<img align="center" width="400" src="https://github.com/gwbres/rinex/blob/main/doc/plots/sv_esbc00dnk.png">


In case Differential context is activated (`--nav`) determining
which vehicules were both sampled in
Ephemeris and Observation Context becomes rapidly mandatory.
Otherwise, we just don't know which vehicule could be a good candidate
for Differential operations.

`--sv-epoch` has a special behavior when `--nav` context is provided,
in this scenario, we exhibit vehicules encountered in both files
accross epochs, and this help decide which one to use later on.

This special behavior will work well if you select a unique vehicule,
otherwise plot becomes rapidly messy:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --nav test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --retain-sv G01 \
    --sv-epoch
```

<img align="center" width="400" src="https://github.com/gwbres/rinex/blob/main/doc/plots/sv_diff_esbc00dnk.png">

With one or two iterations (focus on G01, G08 for instance), we determine that G08 was
seen in both Ephemeris and Observation context, and some interesting processing becomes available.  
To learn more about differential processing, refer to the 
Differential proceesing operations described
[in this page](https://github.com/gwbres/rinex/blob/main/rinex-cli/doc/processing).
