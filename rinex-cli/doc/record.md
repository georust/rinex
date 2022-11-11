Record analysis
===============

RINEX files are huge, complex and vary a lot.   
With these tools, we aim at providing an easy to use and efficient interface
to manipulate and visualize RINEX record.


Observation RINEX
=================

When analyzing Observation RINEX, one plot per kind of observations
is to be generated

- "phase.png": Phase data
- "pseudorange.png": Pseudo Range data
- "ssi.png": Signal Strengths
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

Now let's analyze all observations for these vehicules:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --retain-sv G21,G09,G27 \
    -w "2020-06-25 00:00:00 2020-06-25 03:00:00"
    --plot
```

Here's the resulting plot containing doppler shifts for instance:

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_gpsdoppler.png">

When you see black symbols on the phase plot, it means a cycle slip
may have happen at this epoch (due to temporary loss of lock).

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_cycleslip1.png">

Cycle slips may happen randomly, for a given channel and signal.   
In `ESBC00DNK`, apparently no GPS vehicules declare a possible cycle slip.  
And here we see that only R09(L3) seems affected.  

To learn more about cycle slips, refer to the [processing page](processing.md).
