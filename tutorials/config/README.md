Config scripts
==============

Configuration files used in the tutorials. They can also serve as reference point
prior further customization.

- [Survey](./survey) are configuration dedicated to static geodetic surveying.
In this application, we want to determine the coordinates of a single and static GNSS receiver
with highest precision and without a priori knowledge.

## Notes on PVT Solver configurations

For each script we emphasize the most impacting criterias like
the [Solving strategy](https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html) and the type of
filter being used.

All physical phenomena are modelized by default, making these scripts more "high precision" oriented.  
You can easily modify that to see how a particular phenomenom impact the solutions.

By default we use express the solutions in GPST, 
but [any supported Timescale](https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.TimeScale.html) applies. 

## More information

Refer to the [Rinex Wiki](https://github.com/georust/rinex/wiki) for positioning and other examples.  
Refer to the [GNSS-RTK API](https://docs.rs/gnss-rtk/latest/gnss_rtk/) for indepth information on available settings.
