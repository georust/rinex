RINEX Preprocessing
=================== 
  
It is important to master the preprocessing filer designer to operate this tool efficiently.

Several algorithms are known:  

* `mask` for [mask filters](#masking-operations): to focus or get rid of specific data subsets
* `decim` for [record decimation](#data-decimation): to get rid of specific epochs (as is)
* `smooth` for [smoothing filters](#smoothing-filters)
* `interp` for [interpolation filters](#interpolation-filters)

Preprocessing operations and algorithms are summoned with `-P`.
You can request as many ops as you want, for example :

```bash
rinex-cli \
    -P GPS -P ">G08" \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
```

Faulty preprocessor description will not cause the application to crash, 
they simply do not apply (check the logs).

All supported preprocessing operations can either apply to the entire data set
or only to a specified data subset, see [this paragraph](#advanced-data-subsets) for more information.

## Masking operations

Use Mask filters to focus on data you are interested in, or get rid of entire data subsets.

As mask filter is one operand and a mask to apply to a particular kind of data.   

### Mask Operands

List of supported Mask Operands:

* Lower Than (<) 
* Lower Than or Equals (<=) 
* Greater Than (>)
* Greater Than or Equals (>=)
* Equals (=)
* Ineq (!=)

Example:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P !=G08,G09,G10
```

When the operand is omitted, _Equals_ (=) operand is implied.  
For example, here we retain vehicles G08 and R03 only.

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P G08,R03
```

Refer to the MaskFilter API in the RINEX official documentation for more
advanced mask filters.

## Epoch filter
  
Any valid Hifitime::Epoch string description is supported.  

For example, this mask will strip the record to a unique _Epoch_
because _Equals()_ operand is implied. We need inverted commas to represent
the correct datetime (UTC):

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P "2020-06-25T04:00:00 UTC" -P ">G08"
```

Define a time window with something like this:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P ">2020-06-12T08:00:00 UTC" \
    -P "<=2020-06-25T16:00:00 UTC" \
    -P ">G08"
```

## SV filter

A comma separated list of Sv (of any length) is supported.  
For example, retain _R03_ and _E10_ with the following:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P R03,E10
```

`Sv` target is the only one amongst CSV arrays that supports more than "=" or "!=" operands.   
This is used to filter on SV PRN. 
For example here we can select PRN above 08 for GPS and below (included) 10 for Galileo:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P ">G08" "<=E10"
```

## Constellations filter

Retain specific constellations. For example we only retain GPS with this:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P GPS
```

Use CSV to describe several constellations at once:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P GPS,BDS
```

Inequality is also supported. For example: retain everything but Glonass

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P !=GLO
```

SBAS is a special case. If you use simply "SBAS", you can retain or discard
SBAS systems, whatever their actual constellation. For example we
retain all GPS and any SBAS with this:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz -P GPS,SBAS
```

If you want to retain specific SBAS, you have to name them precisely, we support all of them
(see Constellation module API). For example, retain GPS, EGNOS and SDCM with this:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz -P GPS,EGNOS,SDCM 
```

Note that the following `teqc` equivalent filters are also supported.

- `-G` removes GPS (equivalent to `-P !=GPS`)
- `-C` removes BDS
- `-E` removes Galileo
- `-R` removes Glonnass
- `-J` removes QZSS
- `-S` removes all SBAS vehicles

If you want to remove specific SBAS constellations, for example EGNOS, you have to use
`-P`:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz -P !=EGNOS
```

## Observables

A comma separated list of Observables is supported.  

Here for example, we retain phase observables on both L1 and L2 frequencies,
notice this is once again case insensitive:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P L1C,l2c
```

## Navigation frames

It is possible to retain only certain types of Navigation Broadcast frames. 

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P eph,ion
```

## Navigation Messages

A comma separated list of Navigation Messages is supported.  

For example, with the following, we are only left with legacy messages

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P lnav
```
 
## Elevation mask

The "e" prefix is used to describe an elevation mask.  
Currently, an Elevation Mask can only apply to NAV RINEX.

For example, with the following mask we retain all vehicles with
an elevation angle above 10°:

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P "e> 10.0"
```

Combine two elevation masks to create an elevation range condition:

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P "e> 10.0" \
    -P "e <= 45"
```

## Azimuth mask

Use the prefix "a" for an Azimuth angle mask (follow the Elevation Mask procedure).

## Decimation filter

One preprocessing algorithm is record _decimation_ to reduce
data quantity or increase sampling interval. It is described with `decim:`. 

### Decimate by a ratio

Decimate an entire record to reduce the data quantity.

For example, decimate by 4 and zoom on a portion of the day:
we now have 2 minutes in between two data points.

```bash
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P decim:4 \
    -P ">2020-06-25T08:00:00 UTC" \
    -P "<=2020-06-25T10:00:00 UTC"
```

### Decimate to match a duration

Decimate this record to increase the epoch interval (reduce the sample rate)
to it matches 10 minutes. In this example, this will apply to only a specific time frame:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P "decim:10 min" \ # whitespace, to describe a valid duration
    -P ">2020-06-25T08:00:00 UTC" \
    -P "<=2020-06-25T10:00:00 UTC"
```

### Advanced RINEX context

Algorithms like decimation, interpolation and smoothing either
apply to an entire set or a speficic subset.

In this example, we load both Observations and Navigation frames,
and retain only Phase L1 + L2 observables from the observations, and decimate
them by 4:

```bash
rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    --nav test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -P L1C,L2C \
    -P decim:4:L1C
```

### Observation targetted ops

In RTK or QC mode, you probably want to load a super set of Observations
and Navigation frames. Most of the time Observations come with a fast
sample rate and are pretty lengthy. 
In order to target only Observations in the data context, we allow "observ:"
to prefix all -P operations, to specify the should only apply to Observations.

For example let's consider we have a superset of daily data that comprise 
two observations spanning 24h and a bunch of Navigation broadcast :

```bash
rinex-cli \
    -r \
    -d DATA/2023/256/OBS \
    --nav DATA/2023/256/NAV \
    --nav DATA/2023/256/SP3
```

Solving RTK in this case if probably not meaningful, let's decimate by 
3600 so we resolve only a few positions

```bash
rinex-cli \
    -r \
    -d DATA/2023/256/OBS \
    --nav DATA/2023/256/NAV \
    --nav DATA/2023/256/SP3 \
    -P observ:decim:3600
```

### Hatch smoothing special case

The Hatch smoothing algorithm is a special case, considering it only applies to
Code Pseudo Range it can obviously only apply to Observation frames.

For example, here we load a set of two files, only the L1 + L2 phase observations
from the CRINEX are to be smoothed.

```bash
rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    --nav test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -P L1C,L2C \
    -P smooth:hatch:c1c 
```
