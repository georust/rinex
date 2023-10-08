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
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
```
  
Any amount of preprocessing algorithm can be stacked:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P L1C G08,G09,G10
```

In any case, invalid descriptors will not crash the app but only generate an error trace.

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

## Stacked preprocessing ops

A whitespace separates two preprocessing operations (ie., two sets of CSV).
Therefore it is considered as two separate filters. For example here, we're only left with
G08 and R03 data.

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P GPS,GLO,BDS G08,R03  
```

If one opeation requies a whitespace, it needs to be wrapped
in between inverted commas. Most common example is the [Epoch](epoch-target) description.

## Epoch filter
  
Any valid Hifitime::Epoch string description is supported.  

For example, this mask will strip the record to a unique _Epoch_
because _Equals()_ operand is implied:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P "2020-06-25T04:00:00 UTC" GPS >G08 # notice the \" due to whitespace requirement
```

Use two epoch masks to zoom in on 
the ]2020-06-12 08:00:00 ; 2020-06-12 10:00:00] time window:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P ">2020-06-12T08:00:00 UTC" "<=2020-06-25T16:00:00 UTC" GPS >G08
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
    -P >G08 "<=E10"
```

## Constellations filter

Retain specific constellations. For example we only retain GPS with this:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P GPS
```

You can stack as many filters as you want, using csv. For example, retain
BeiDou also:

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

SBAS is a special case. If you use "SBAS", you can retain or discard
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

### Decimate by a ratio

Decimate an entire record to reduce the data quantity.

For example, decimate by 4 and zoom on a portion of the day:
we now have 2 minutes in between two data points.

```bash
./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P decim:4 ">2020-06-25T08:00:00 UTC" "<=2020-06-25T10:00:00 UTC"
```

### Decimate to match a duration

Decimate this record to increase the epoch interval (reduce the sample rate)
to it matches 10 minutes:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P "decim:10 min" \ # whitespace once again
        ">2020-06-25T08:00:00 UTC" "<=2020-06-25T10:00:00 UTC"
```

### Advanced: data subsets

Algorithms apply to the entire record by default, but you can also specify
to apply it only a subset of the RINEX record.  
This is done by adding `:X` after any preprocessing algorithm (like decimation for example)
where X is a valid data Mask as previously defined.

For example, here we extract L1C and L2C phase data
on GPS constellation (PRN >= 08), but we reduce the L1C observation
quantity by 4:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P GPS ">=G08" L1C,L2C "decim:4:L1C"
```

With the following command line, we retain L1C L2C observations
only for Vehicle >08 <25 of the american constellation.
Entire set is decimated by 2 (reduces record quantity by 50%).  
L1C observations are decimated by another factor of (total reduction is 75%)
only for these observations.


```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P GPS ">=G08" L1C,L2C decim:2  decim:2:L1C
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
