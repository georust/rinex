User context and analysis
=========================

Based on the provided context, some operations are feasible or not.  
Also, it is important to understand what files are mandatory for the analysis
you want to perform.

## Meteo RINEX analysis

To analyze a Meteo file, a primary file of this type should be passed
to `--fp` (or `-f`). In this case, you get a visualization
of all observations. An in depth statistical analysis is feasible with `--qc`.

It does not make sense to stack other RINEX files to such analysis.

## IONEX analysis

To analyze a IONEX file, a primary file of this type should be passed
to `--fp` (or `-f`). In this case, you get a world map visualization
of the provided TEC map. Unfortunately we can only visualize the TEC map
at a single epoch, because we cannot animate the world map at the moment.
Therefore, it makes sense to zoom in on the Epoch you're interested in,
with the proper `-P` preprocessor command. Refer to related section.

## OBSERVATION Data analysis

To analyze OBSERVATION Data, a primary file of this type should be passed
to `--fp` (or `-f`). Like Meteo analysis, all physical observations can be visualized
and an in depth statistical analysis can be also performed with  `--qc`.  

Example of SSI visualization :

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/esbc00dnk_ssi.png">

OBS RINEX analysis can be augmented with NAVIGATION Data, by providing
one file of this type with `--nav`.  
In this case, all feasible [analysis](NavigationDataAnalysis) that apply to this file
will also be performed.

Providing Navigation data gives the ability to determine elevation angle.  
The Elevation Angle is added on top of SSI Observations to correlate both.  

In case of `--qc` analysis, elevation masking becomes feasible and
data is reported along such information.

When invoking the preprocessor with `-P`, conditions apply to all context augmentations.  
For example, if you request an elevation mask, it gets applied to the NAV file,
and also to the OBS file.

Of course, filter conditions that apply to a specific type of RINEX,
like LLI masking for OBS RINEX, or Navigation Type frame filter, will only apply
to those RINEX files.

## NAVIGATION Data analysis

To analyze NAVIGATION Data, a primary file of this type should be passed
to `--fp` (or `-f`). With this type of file, a skyplot view to visualize SV
in the sky along the day course is drawn. SV embedded clock data is also visualized
(both Clock bias and Clock Drift).

Example of SV clock data visualization :

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/sv_clocks.png">

Skyplot view example : 
<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/skyplot.png">

## SP3 file analysis

This tool allows adding one SP3 file with `--sp3`, 
to provide high precision Orbit and Clock behavior predictions.  
Ideally, the user should provide SP3 file that covers the entire time frame
of the RINEX context.  
When SP3 context is provided, a visualization of both Clock & Orbit data is enabled.  
The SP3 context is interpolated to match the RINEX context, this is emphasized on the plot.  

Providing SP3 context will then enable Single Precise Positioning (SPP) in near future.
