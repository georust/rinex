Config scripts
==============

This folder contains a set of configuration files (usually JSON structure description),
to customize the behaviors of our applications. Currently, RINEX-Cli is the only application
that accepts a config script.

- [Survey](./survey) is a set a configuration dedicated to static geodetic surveying.
In this application, we want to determine the coordinates of a single and static GNSS receiver
with highest precision and without apriori knowledge.
- [QC](./qc) is a set a configuration dedicated to data quality check in RINEX-Cli.
This mode is in standby and will evolve in near future. We're currently focused on Surveying and RTK.

## Notes on PVT Solver configurations

For each script we emphasize the [Solving strategy]() and the type of navigation filter being used.   
All physical phenomena are modelized by default, making these scripts more "high precision" oriented.  
You can easily modify that to see how a particular phenomenom impact the solutions.

By default we use express the solutions in GPST, but [any supported Timescale is valid]().  

possibly used to obtain the clock offset.

## More information

Refer to the [Rinex Wiki]() for positioning and other examples.  
Refer to the [GNSS-RTK]() solver API for more information on the solver configuration. 
