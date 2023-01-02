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
    -P mask:G08,G09,G10
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
```
  
Any amount of preprocessing algorithm can be stacked:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P mask:L1C mask:G08,G09,G10
```

## Masking operations

Use Mask filters to focus on data you are interested in, or get rid of entire data subsets.

As mask filter is one operand and a mask to apply to a particular kind of data.   

### Mask Operands

List of supported Mask Operands:

* `lt` Lower Than (<) 
* `leq` Lower Than (<=) 
* `gt` Greater Than (>)
* `geq` Greater Than (>=)
* `eq` Equals Than (=)
* `neq` Lower Than (!=)

Example:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:neq:G08,G09,G10
```

When the operand is omitted, the _Equals_ (=) operand is implied

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:G08,G09,G10
```

## Mask Targets

A mask can apply to most major RINEX subsets

## Epoch target
  
Any valid Hifitime::Epoch string description is supported.  
For example, this mask will strip the record to a unique _Epoch_.  
Record would become empty if it does not exist

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:2020-06-12 08:00:00
```

Use a different operand to grab a portion of the day.  
The following mask retains the last 16hours of that file:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:gt:2020-06-12 08:00:00
```

For example, use two epoch masks to zoom in on 
the ]2020-06-12 08:00:00 ; 2020-06-12 10:00:00] time window:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:gt:2020-06-12 08:00:00 mask:leq:2020-06-12 10:00:00 
```

## Duration target

## Sv target

A comma separated list of Sv is supported.  
For example, with the following, we are left with data from _R03_ and _E10_

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:G08,R03,E10
    -P mask:neq:R03,E10
```

The `sv` target supports more than "=" or "!=". With this command for example,
we are left with PRN above 03 for GPS and below 10 (included) for Galileo

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:gt:G08
    -P mask:leq:E10
```

## GNSS target

A comma separated list of Constellation is supported.  
For example, with the following, we are left with data from Glonass and GPS  

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:neq:bds
    -P mask:GPS,GLO
```

## Carrier signals

A comma separated list of Carrier signals is supported.  
For example, with the following, we are only left with observations against L1 and L5

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:l1,l5
```

Carrier signals are one of the exceptions that support more than`eq` and `neq` operands.  
For example, with the following we retain L2, L5 signals and L1 is excluded:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:gt:l1
```

## Observables

A comma separated list of Observables is supported.  
For example, with the following, we are only left with phase observations,
against L1 and L2 carriers 

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P mask:L1C,l2c
```

## Navigation frames

A comma separated list of Navigation Frames is supported.  
For example, with the following, we are only left with Ephemerides

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P mask:eph
```

## Navigation Messages

A comma separated list of Navigation Messages is supported.  
For example, with the following, we are only left with legacy messages

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P mask:lnav
```
 
Navigation Messages is one of those exceptions that support more that "=" (`eq`) or "!=" (`neq`) operands.  

With the following mask, we retain only modern navigation messages

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P mask:gt:lnav
```

### Unfeasible operations
  
Some data targets do not support all operands.  
For example, it is impossible to use the (>) or (<=) operand on `gnss:` or `obs:` categories,  
because it does not make sense.    
  
Exceptions exist for `Sv` and `NavMsg` targets.   

### Filter description

The description is case insensitive: `mask:R15` is the same as `mask:eq:r15`.  
  
The description is whitespace tolerant, but you then need inverted commands when using the command line:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P 'mask: gt: G08, R13' \
    -P 'mask: leq: R15'
```

## Elevation mask

## Decimation filter

One preprocessing algorithm is record _decimation_ to reduce
data quantity or increase sampling interval. It is described with `decim:`. 

### By a ratio

Decimate an entire record to reduce the data quantity by 2 (-50%)

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P 'decim:2'
```

### By an interval

Decimate this record to increase the epoch interval (reduce the sample rate)
to it matches 10 minutes:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P 'decim:10 minutes'
```

### Advanced: Decimate a data subset

Algorithms apply to the entire record by default, but you can specify
to apply it only a subset.
Subsets are described like Data Masks previously defined.

Decimate L1C observations by a factor of 10: 

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P 'mask:L1C,L2C' 'decim:2:l1c'
```

Now open the `graphs.html` report and see how the L1C graph differs from the L2C graph.

This applies to any filter opmodes. For example, lets reduce the L1C rate
by 2 with the following command

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P 'mask:L1C,L2C' 'decim:1 min:l1c'
```

## Advanced: Hatch Smoothing Filter

If you are working on Pseudo Range observations (only?) but want to reduce
the noise they come with, the Hatch filter algorithm is a standard solution to that problem.  
The hatch smoothing filter is requested with `smooth:hatch` and can be applied either
to all Pseudo Range observations or specific observations.

For example, compare the smoothed L1C observations to noisy L2C observations,
after the following command

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.gz \
    -P 'mask:L1C,L2C' 'smooth:hatch:l1c'
```
