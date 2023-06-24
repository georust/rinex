IONEX
=====

IONEX files can be passed as primary RINEX with the `--fp` argument.

During such analysis we plot the TEC maps ontop of a world map.  
The TEC map represent the total electron density at a given location of the ionosphere.

Limitations
===========

Some limitations exist to this day.

We cannot animate the plot, so it is impossible to depict several epochs in a single plot.
It is recommneded to zoom in on a specific epoch to plot the desired density.

In this example, we plot the TEC map for the last epoch of this file: 

```bash
target/release/rinex-cli \
	--fp test_resources/IONEX/V1/jplg0010.17i.gz \
	-P "2017-01-01T20:00:00 UTC" 
```

RMS error map cannot be plotted at the moment.

3D IONEX cannot be visualized at the moment, we have no mean to display TEC maps for different altitudes.
