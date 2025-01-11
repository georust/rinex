Config scripts
==============

This toolbox uses `JSON` configuration scripts to operate more precisely.

When no configuration script is passed: we rely on the default script.  
The default script is stored in this very folder, which might be a starting point
for your, or help you understand the default behavior.

- [documentation.json (DO-NOT-USE)](./documentation.json) is
a fully documented (one explanation per field) `rinex-cli --cfg`uration script.
Because Json do not allow comments, this file cannot be used directly.

- [prefered-SP3.json](./prefered-SP3.json) is a toolbox configuration
that has SP3 prefered over Broadcast Radio Navigation. It is sometimes
used in our tutorials

- [Custom signal preferences](SIGNALS/README.md) are simple
configurations that apply to signal quality control, reporting
and analysis. For example, this contains a config script
that enables all possible signal analysis.

- [Surveying confugurations](./SURVEY/README.md) that may apply
to precise geodetic applications. Static surveying without apriori knowledge
for example, may apply to RTK reference station calibrations. This also
covers CGGTTS applications that fall in the static surveying category.

- [RTK](./rtk) are configuration scripts dedicated to 2D (single base single rover)
differential positioning. Use them in conjonction of our [RTK examples](https://github.com/georust/rinex/main/tree/tutorials).
They apply to the `rtk --cfg`uration option. It is up to you to load valid
data and define a correct data set, along this configuration; otherwise RTK will not operate correctly.

## Notes on PVT Solver configurations

For each script we emphasize the most impacting criterias like
the [Solving strategy](https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html) and the type of
filter being used.

The solver models everything by default. These examples are usually "high precision" oriented.
You can easily modify that to see how a particular phenomenom impact the solutions.

By default we use express the solutions in GPST, 
but [any supported Timescale](https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.TimeScale.html) applies. 

## More information

Refer to the [Rinex Wiki](https://github.com/georust/rinex/wiki) for positioning and other examples.  
Refer to the [GNSS-RTK API](https://docs.rs/gnss-rtk/latest/gnss_rtk/) for indepth information on available settings.
