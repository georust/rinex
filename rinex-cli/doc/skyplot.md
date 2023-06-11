Skyplot
=======

A skyplot view is requested with `-y`.  

It is mandatory to provide Navigation Data to be able to access
the skyplot view.

The skyplot view exhibits the observed vehicles, accross epochs,
in terms of azimuth and elevation angles (position in the sky).

The first epoch where a vehicle appears is always depicted with a circle symbol.  
The last epoch before a vehicle dissapears is always depiceted with a square symbol.

If only Navigation data is provided (`--fp`), 
the curve color emphasizes the epoch spanning:

- colder color means older data
- hotter color means newer data

Here is an example of a simple skyplot view

```bash
rinex-cli -y --retain-sv G08,G18,R19,R03 \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz
```

Data filters and resampling still applies to the skyplot view, 
and this view can also be stacked to record analysis or other calculations / visualizations.

If user provides Observation data, the color gradient now emphasizes
the received signal power. It is usually indexed on the elevation angle, 
so it is natural to get hotter data points in the center of the sky view.

Here is an example of "enhanced" skyplot view,
where both Navigation data and Observations were provided 

```bash
rinex-cli -y --retain-sv G08,G18,R19,R03 \
    --nav test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
```
