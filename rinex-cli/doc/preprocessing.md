RINEX Preprocessing
=================== 
  
It is important to master the preprocessing filer designer to operate this tool efficiently.

Several algorithms are known:  

* `mask` for [mask filters](#masking-operations): to focus or get rid of specific data subsets
* `decim` for [record decimation](#data-decimation): to get rid of specific epochs (as is)
* `smooth` for [smoothing filters](#smoothing-filters)
* `interp` for [interpolation filters](#interpolation-filters)

A preprocessing algorithm is described with a string and passed with `-P`,

for example:

```bash
rinex-cli \
    -P G08,G09,G10
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
```
  
Any amount of preprocessing algorithm can be stacked:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P L1C G08,G09,G10
```

In any case, invalid descriptors will not crash the app but only generate an error trace.

## Masking operations

Use Mask filters to focus on data you are interested in, or get rid of entire data subsets.

As mask filter is one operand and a mask to apply to a particular kind of data.   

### Mask Operands

List of supported Mask Operands:

* Lower Than (<) 
* Lower Than (<=) 
* Greater Than (>)
* Greater Than (>=)
* Equals Than (=)
* Lower Than (!=)

Example:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P !=G08,G09,G10
```

When the operand is omitted, _Equals_ (=) operand is implied

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P G08,R03
```

Refer to the MaskFilter API in the RINEX official documentation for more
advanced mask filters.

## Stacked preprocessing ops

A whitespace separates two preprocessing operations.

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P GPS,GLO G08,R03  
```

Therefore, if a filter operation involves a whitespace, it requires to be wrapped
in between inverted commas. Most common example is the [Epoch](epoch-target) description.

### Targetted subsets

Most RINEX data subsets can be targetted by a mask filter. 

## Epoch target
  
Any valid Hifitime::Epoch string description is supported.  

For example, this mask will strip the record to a unique _Epoch_
because _Equals()_ operand is implied:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P ">2020-06-25T04:00:00 UTC" GPS >G08 # notice the \" due to whitespace requirement
```

Retain a single Epoch with _Equals()_ ops. Equals is the implicit
operand if the operand is omitted:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P "2020-06-25T04:00:00 UTC" GPS >G08 # notice the \" due to whitespace requirement
```

For example, use two epoch masks to zoom in on 
the ]2020-06-12 08:00:00 ; 2020-06-12 10:00:00] time window:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P ">2020-06-12T08:00:00 UTC" "<=2020-06-25T16:00:00 UTC" GPS >G08
```

## Duration target

TODO

## Sv target

A comma separated list of Sv is supported.  
For example, with the following, we are left with data from _R03_ and _E10_

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P R03,E10
```

The `sv` target supports more than "=" or "!=". With this command for example,
we are left with PRN above 03 for GPS and below 10 (included) for Galileo

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P >G08 "<=E10"
```

## Constellations

A comma separated list of Constellations is supported.  
For example, with the following, we are left with data from Glonass and GPS  

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P !=BDS GPS,GLO # ineq(BDS) AND eq(GPS,GLO)
```

## GNSS Signals

A comma separated list of Carrier signals is supported.  
For example, with the following, we are only left with observations against L1 and L5

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P "l1,l5"
```

## Observables

A comma separated list of Observables is supported.  
For example, with the following, we are only left with phase observations,
against L1 and L2 carriers. As always, most commands are case insensitive,
to help the user form them easily:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P "L1C,l2c"
```

## Navigation frames

A comma separated list of Navigation Frames is supported.  
For example, with the following, we are only left with Ephemerides

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P eph
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

## Orbit fields

When parsing NAV RINEX, focus or discard orbits you're not interested by
using official orbit fields as the filter payload.

For example, with the following we only retain "iode" and "crs" fields, because _Eq()_ is implied:

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P "iode,CRS" \
```

Notice once again that this is case unsensitive.

## Decimation filter

One preprocessing algorithm is record _decimation_ to reduce
data quantity or increase sampling interval. It is described with `decim:`. 

### By a ratio

Decimate an entire record to reduce the data quantity.

For example, decimate by 4 and zoom on a portion of the day:
we now have 2 minutes in between two data points.

```bash
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P decim:4 ">2020-06-25T08:00:00 UTC" "<=2020-06-25T10:00:00 UTC"
```

### By an interval

Decimate this record to increase the epoch interval (reduce the sample rate)
to it matches 10 minutes:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P "decim:10 min" \ # whitespace once again
        ">2020-06-25T08:00:00 UTC" "<=2020-06-25T10:00:00 UTC"
```

### Advanced: Use data subsets

Algorithms apply to the entire record by default, but you can specify
to apply it only a subset.
Subsets are described like Data Masks previously defined.

For example, here we retain both L1C and L2C phase data
on GPS constellation (PRN >= 08). But we reduce the quantity
of L1C observations by 4:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P GPS ">=G08" L1C,L2C "decim:4:l1c"
```

## Advanced: Hatch Smoothing Filter

If you are working on Pseudo Range observations but want to reduce
the noise they come with, the Hatch filter algorithm is a standard solution to that problem.  
The hatch smoothing filter is requested with `smooth:hatch` and can be applied either
to all Pseudo Range observations or specific observations.

For example, compare the smoothed L1C observations to noisy L2C observations,
after the following command

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P 'L1C,L2C' 'smooth:hatch:l1c'
```
