Position solver
===============

The position solver is currently an "advanced" SPP solver. 
SPP stands for Single Frequency Precice Point solver which means
you get a precise point location (ideally with metric accuracy) for a minimal
- down to single frequency data context.

When we say "advanced" SPP it means it supports more than the minimal prerequisites
for a pure SPP solver. For example it is possible to still require SPP solving
but use other criteria that makes it a little closer to PPP.

Command line interface
======================

* use `-p` to request position solving.
From the provided data context, we will try to evaluate the user position
the best we can

* use `--spp` to force to SPP solving.

* `--ppp` to force to PPP solving. It exists but not entirely supported to this day.

Minimal data context
====================

A minimum of one primary RINEX Observation file with broadcast Ephemeris
valid for that particular time frame is required.

SP3 can be stacked, broadcast Ephemeris are still required, we will prefer SP3
for certain steps in the solving process.

Example of minimum requirement :

```bash
./target/release/rinex-cli -P GPS,GLO --spp \
    --fp DATA/2023/OBS/256/ANK200TUR_S_20232560000_01D_30S_MO.crx \
    --nav DATA/2023/NAV/255 \
    --nav DATA/2023/NAV/256
```

Example of SP3 extension :

```bash
./target/release/rinex-cli -P GPS,GLO --spp \
    --fp DATA/2023/OBS/256/ANK200TUR_S_20232560000_01D_30S_MO.crx \
    --nav DATA/2023/NAV/255 \
    --nav DATA/2023/NAV/256 \
    --sp3 DATA/2023/SP3/255 \
    --sp3 DATA/2023/SP3/256
```

Position solver and results
===========================

The solver will try to resolve the navigation equations for every single Epoch
for which :

* enough raw GNSS signals were observed in the Observation RINEX
* enough SV fit the Navigation requirements
* all minimal or requested models were correctly modelized

The solver can totally work with its default configuration, as long as the previous points stand.
But you need to understand that in this configuration, you can't hope for an optimal result accuracy.

Mastering and operating a position solver is a complex task.  
To fully understand what can be achieved and how to achieve such results,
refer to the [gnss-rtk](../gnss-rtk/README.md) library documentation.
