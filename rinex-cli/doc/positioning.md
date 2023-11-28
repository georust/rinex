Positioning
===========

A positioning mode exists and will resolve the receiver's location.  
By default, this mode is not activated even when you provide a compatible RINEX context.

Use `--spp` to activate position solver.   
Combine `-p`  to disable any other record analysis and focus on position solving.

## Position solving

Position solving is quite complex and requires deep knowledge to achieve the best results.   
This documentation will only focus on how to operate the RINEX post processor to operate the solver.  
To understand the inner workings, refer to the [rtk-rs documentation](https://github.com/rtk-rs/gnss-rtk).

This will guide you on :

* existing solving strategies and methods
* current limitations
* physical effects that are taken into account
* environmental effects: how they can be modeled and how to model them yourself
* atmospherical effects: how they can be modeled and how to model them yourself
* how to optimize the configuration and solving strategy to achieve the best precision

## Understand the requirements

Position solving requires a RINEX context that is consistent with the solving strategy.

It will always required both Observation (for raw signals) and Navigation RINEX (for
ephemerides) at least. This application's interface is powerful enough to easily
overlap Observation data by Navigation data easily. 

## Take advantage of the applications log

The RTK solver and its dependencies, make extensive use of the Rust env. logger.  
The env logger is managed with `${RUST\_LOG}` and we emphasize:

- Epochs and elected vehicles
- solving strategy
- signals, SNR and state vectors condition
- phyiscal phenomena taken into account
- modelled environmental perturbations

the logger streams to stdout by default but that can be redirected easily.

## Typical mistakes

If your post processing workflow does not generate any PVT solutions, the provided
RINEX context is most likely not sufficient. Check out the logs to determine at which
point the solving process is "failing". The solver is quite verbose, if you know the follwing
few points, you instantly know what is missing.

The position solver will not converge (for a given Epoch) as long as it failed to gather more than 3 vehicles for that desired Epoch.

* Missing Broadcast Navigation : Navigation Data will always be required,
even though you provided overlapping and high quality SP3.

* Too many undetermined ephemerides: 

``` bash
[2023-11-03T07:45:19Z WARN  rinex_cli::positioning::solver] 2023-09-13T02:44:00 GPST (G27) : undetermined ephemeris
```

We're trying to resolve using G27 @ 2023-09-13T02:44:00 GPST but the ephemerides are undetermined.    
That means the Navigation data is missing for that time window: either not provided, or unexpected gap.   

* Too many interpolation failures

The solver needs to interpolate each individual SV state vector for each individual Epoch.  

```bash
[2023-11-03T08:03:57Z WARN  gnss_rtk::solver] 2023-09-13T02:43:59.917986109 GPST (G31) : interpolation failed
```

We're trying to resolve @ 2023-09-13T02:43:59.917986109 using G31 but we failed to interpolate its state vector.  
If you provide SP3, you should only get a very tiny fraction of such "errors".  
The error comes from data gaps that make the interpolation method fail. The data must be steady especially
the higher the interpolation order you use.

## Ionospheric delay

TODO 

IONEX is not taken into account by the solver at the moment.

## Tropospheric delay

If you have access to Meteo RINEX from the day the signals were sampled, we highly encourage loading them.  
The solver will then prefer this data source as the source of tropospheric delay components.

## Command line examples

Focus on --spp solver using GPS.  
Note that we import data sampled by a unique station (ANK200TUR) on day 256 of year 2023.  
We overlap that day with ephemerides and SP3 easily. A single SP3 file is used, but its enough to resolve
many solutions.

```bash
./target/release/rinex-cli --spp -p -P GPS \
    -f DATA/2023/OBS/256/ANK200TUR_S_20232560000_01D_30S_MO.crx.gz \
    -d DATA/2023/NAV/256 \
    -f DATA/2023/SP3/255/USN0OPSULT_20232551800_02D_15M_ORB.SP3.gz

[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/IZMI00TUR_S_20232560000_01D_JN.rnx.gz"
[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/KRS100TUR_S_20232560000_01D_GN.rnx.gz"
[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/ANK200TUR_S_20232560000_01D_JN.rnx.gz"
[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/ANK200TUR_S_20232560000_01D_CN.rnx"
[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/ANK200TUR_S_20232560000_01D_RN.rnx"
[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/IZMI00TUR_S_20232560000_01D_GN.rnx.gz"
[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/ANK200TUR_S_20232560000_01D_CN.rnx.gz"
[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/ANK200TUR_S_20232560000_01D_GN.rnx.gz"
[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/ANK200TUR_S_20232560000_01D_EN.rnx.gz"
[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/ANK200TUR_S_20232560000_01D_EN.rnx"
[2023-11-03T08:13:08Z TRACE rinex::context] loaded brdc nav "DATA/2023/NAV/256/ANK200TUR_S_20232560000_01D_RN.rnx.gz"
[2023-11-03T08:14:40Z TRACE rinex::context] loaded observations "DATA/2023/OBS/256/ANK200TUR_S_20232560000_01D_30S_MO.crx.gz"
[2023-11-03T08:14:40Z TRACE rinex::context] loaded sp3 "DATA/2023/SP3/255/USN0OPSULT_20232551800_02D_15M_ORB.SP3.gz"
```

## Configuration file

Use `--cfg`.

TODO


## PVT solutions

Solutions are always written into a CSV file, within your workspace.   
You can activate the generation of a GPX track with `--gpx`.     
You can activate the generation of a KML track with `--kml`.   

The solutions are also plotted and analyzed graphically, opening that view is automatic, unless
you set the `-q` quite option.


One output is the local GNSS receiver time and the dilution of precision on the time component :

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/clk_tdop.png">

The vertical and horizontal dilution of precision are also depicted :

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/hdop_vdop.png">

## Current limitations

Refer to the GNSS solver's limitation, explained in the 
[rtk-rs documentation suite](https://github.com/rtk-rs/gnss-rtk)

