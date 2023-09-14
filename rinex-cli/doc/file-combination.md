User context and analysis
=========================

Based on the provided context, some operations are feasible or not.  
It is important to understand what files you should provided, for the analysis you want to perform.

## OBSERVATION Data analysis

Observation is the main RINEX type for high precision positioning.

To analyze Observation Data, a primary file of this type should be passed
with `--fp` (or `-f`). An in depth analysis can be performed and rendered as HTML with `--qc`.  
All physical observations are rendered in a graph, for example, this is an extract
of SSI :

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/esbc00dnk_ssi.png">

OBS RINEX analysis can be augmented with several other input files:
- Navigation Data files : with `--nav` (once per file)
- SP3 High precision files : with `--sp3` (once per file)
- ATX Data files : with `--atx` (once per file)

Once a certain type of RINEX is added to the context, all analysis
that are usually performed when this file serves as primary data still apply.  
For example, having one `--nav` file unlocks
[Navigation Data analysis](https://github.com/georust/rinex/blob/main/rinex-cli/doc/file-combination.md#navigation-data-analysis)
apply.

Providing Ephemeris data or Orbit files is the most useful to Observation Data analysis.

- The SV elevation angle can be taken into account when processing or reporting
- Elevation masking can now also apply
- Positioning is only possible when a complete coherent context is provided

When invoking the preprocessor with `-P`, conditions apply to all context augmentations.  
For example, if you request an elevation mask, it gets applied to the NAV file,
and also to the OBS file.

Of course, filter conditions that apply to a specific type of RINEX,
like LLI masking for OBS RINEX, or Navigation Type frame filter, will only apply
to those RINEX files.

## Meteo RINEX analysis

Meteo RINEX is similar to Observation Data, it can only be processed as a primary file (`-f`).  
Once again, all physical observations an plotted.  

Unlike Observation Data, it is not possible to augment Meteo data with other RINEX files.

## NAVIGATION Data analysis

Navigation Data analysis is performed once this type of data is added to the context.
Either as the primary data type (`-f`), or stacked as context augmentation (`--nav`).  
Keep in mind you can stack as many Navigation Data files as you want, with the latter.

The first view that gets unlocked is the skyplot view.
SV Orbit trajectories are also depicted, expressed in 3D coordinates, in km ECEF.

SV embedded clock data is rendered :
- onboard clock drift and bias (instantaneous)
- the SV clock offset to its parent timescale is plotted 

Skyplot view example : 
<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/skyplot.png">

Example of SV clock data visualization :

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/sv_clocks.png">

## High Precision Orbit Data (SP3)

SP3 files can also augment Observation Data analysis context.  
SP3 cannot be passed as primary data: they can only serve as context enhancer.

With SP3 basically all analysis performed with Navigation Data are feasible,
but on this high precision data.

NB: combining SP3 and NAV ephermeris is not a problem.
When combined (both provided):

- SP3 is prefered for high precision calculations
- SP3 Orbits and Broadcast Ephemeris are compared to one another by plotting
the residual errors between them. For this plot to be available, you need overlapping
Navigation / SP3 Data (common time frame and SV).

Here's an example of overlapping SP3/NAV Ephemeris residuals analysis 

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/TODO.png">

NB: such a context is not hosted on this repo. You'll have to download similar
joint `--nav` and `--sp3` context yourself.

## IONEX analysis

To analyze a IONEX file, a primary file of this type should be passed
to `--fp` (or `-f`). In this case, you get a world map visualization
of the provided TEC map. Unfortunately we can only visualize the TEC map
at a single epoch, because we cannot animate the world map at the moment.
Therefore, it makes sense to zoom in on the Epoch you're interested in,
with the proper `-P` preprocessor command. Refer to related section.

