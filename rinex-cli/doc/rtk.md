RTK solver
==========

RTK mode is requested with `-r` or `--rtk`.

RTK (position solving) is feasible if you provide at least RINEX Observations
(`-f`) and overlapping RINEX Navigation data (`--nav`).

Currently it is also mandatory to provide overlapping SP3 with `--sp3` but that should be fixed
in near future.

As an example (this dataset is not provided), the most basic command line would look like this,
where observations are imported for day 256 and we combine several NAV/SP3 by lazyili importing entire folders:

```bash
./target/release/rinex-cli -P GPS,GLO -r \
    --fp DATA/2023/OBS/256/ANK200TUR_S_20232560000_01D_30S_MO.crx \
    --nav DATA/2023/NAV/256 \
    --sp3 DATA/2023/SP3/256
```

Current limitations
===================

Several limitations exit to this day and must be kept in mind.

- Glonass and SBAS vehicles cannot be pushed into the pool of eligible vehicles.
Until further notice, one must combine -R and -S to the rtk mode

- We only support GPST, GST and BDT. QZSST is expressed as GPST, and I'm not 100% sure this
is correct. 

- The estimated clock offset is expressed against the timescale for which the Observation file is referenced to.
We don't have the flexibility to change that at the moment. 
So far the solver has only be tested against Observations referenced against GPST.

RTK (only)
==========

Use `-r` (or `--rtk-only`) to disable other opmodes. This gives you the quickest results.

```bash
./target/release/rinex-cli -R -S -r \
    --fp DATA/2023/OBS/256/ANK200TUR_S_20232560000_01D_30S_MO.crx \
    --nav DATA/2023/NAV/256 \
    --sp3 DATA/2023/SP3/256
```

RTK configuration
=================

The solver can be customized, either to improve performances
or improve the final resolution. Refer to the library section
that defines the [RTK configuration](https://github.com/georust/rinex/gnss-rtk/doc/cfg.md)
to understand the physics and what they imply on the end result.

A few configuration files are provided in the rinex-cli/config/rtk directory. 

You can use them with `--rtk-cfg`:

Forced SPP mode
===============

By default the solver will adapt to the provided context and will deploy the best strategy.

You can force the strategy to SPP with `--spp` 

It is possible to use the configuration file, even in forced SPP mode, to improve the end results:

In this scenario, one wants to define Ionospheric delay model

Provide SP3
===========

When SP3 is provided, they are prefered over NAV RINEX.  
Refer to the library documentation [TODO](TODO)

Example of SP3 extension :

```bash
./target/release/rinex-cli -R -S --spp \
    --fp DATA/2023/OBS/256/ANK200TUR_S_20232560000_01D_30S_MO.crx \
    --nav DATA/2023/NAV/255 \
    --nav DATA/2023/NAV/256 \
    --sp3 DATA/2023/SP3/255 \
    --sp3 DATA/2023/SP3/256
```

It is totally possible to combine SP3 to a single frequency context,
or a forced --spp strategy.

Results
=======

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

RTK and logger
==============

The RTK solver and its dependencies, make extensive use of the Rust logger.  
Turn it on so you have meaningful information on what is happening:

- Epochs for which we perform the calculations
- Navigation context evolution
- Results and meaningful information
- More information on the configuration and what can be achieved

The Rust logger sensitivity is controlled by the RUST\_LOG environment variable,
which you can either export or adjust for a single run. `trace` is the most sensitive,
`info` is the standard value.

The output is directed towards Stdout, therefore it can be streamed into a text file for example,
to easily compare runs between them.

