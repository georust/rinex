User context and analysis
=========================

Based on the provided context, some operations are feasible or not.  
It is important to understand what files you should provided, for the analysis you want to perform.

## Observation Data 

Having Observation Data in the provided context will generate a graph
for all observation (if graph generation is allowed), allows QC on that type of data,
and allows RTK resolution.

We recommend using -f to load your Observation Data individually.  
If you use -d (recursive loader), you most likely want to load data sampled by identical receivers (for example: 2 days of data).

For example, this is how we represent SSI observations in graphical mode:

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/esbc00dnk_ssi.png">

Special filters like LLI Masking, or Hatch Code filter only apply to Observation data, therefore will leave other part of the context untouched. In RTK, it might be interesting to decimate Observations to reduce the quantity of position fixes. To do so, we allow to only decimate Observations, refer to the preprocessing page for more detail.

## Navigation Data

If broadcast Navigation data is present in the context, we can plot Orbits or embedded clock states if the graphical mode is enable. RTK mode is not feasible if Navigation data is not provided. SV Elevation angle is taken into account in QC reports once Navigation data is provided.

We recommend covering your Observation time frames with Navigation using `-d`.  
Pay extra attention to the apriori position defined in your context, especially if it not defined in the Observation data.

Here's an example of embedded clock visualization :

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/sv_clocks.png">

A skyplot view is also generated in graphical mode, which represents the position in the sky of all SV.

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/skyplot.png">

If broadcast Nav. is provided along SP3, the residual error between them is visualized (in graphical mode).

## SP3 data

Providing SP3 allows us to plot Orbits in graphical mode.  
It also really helps the RTK solving process, considering SP3 have a very reliable and a steady sample rate.  
We highly recommend overlapping your observations with SP3 in RTK mode.

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/sp3_residual.png">

We recommend overlapping your Observation data with SP3 using the `-d` flag.  
The SP3 specs say different data provider should not be mixed together, but we actually tolerate that.  
You might be interested in only loading SP3 using similar mapping functions, in your context.

Once again, running `-i` might help figuring things out.

## Meteo Data 

Meteo RINEX is similar to Observation Data. If present, we can visualize all observations contained
in the context. In QC mode, this part of the context is also taken into account.

In RTK, we recommend providing local Meteo Observations (within 15° latitude difference) from the receiver,
for optimum tropospheric delay compensation.

We recommend overlapping your Observation Data with Meteo Data using `-d`.

## IONEX Data 

Providing IONEX Data allows the visualization of the TEC map (in graphical mode).

We recommend overlapping your Observation Data with IONEX using `-d`.  

In RTK, we recommend covering all daily observations with IONEX for optimum ionospheric delay compensation.

Just like SP3, you most likely want to combine IONEX from the same data provider (identical mapping functions).

TEC map visuzalition :

<img align="center" width="450" src="https://github.com/georust/rinex/blob/main/doc/plots/tec.png">
