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

Use Mask filters to focus on data you're interested in, or get rid of entire data subsets.

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
    -P 'mask:2020-06-12 08:00:00'
```

Use a different operand to grab a portion of the day.  
The following mask retains the last 16hours of that file:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P 'mask:gt:2020-06-12 08:00:00'
```

## Duration target
TODO

## Sv target

A comma separated list of Sv is supported.  
For example, with the following, we're left with data from _R03_ and _E10_

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P 'mask:G08,R03,E10'
    -P 'mask:neq:R03,E10'
```

The `sv` target supports more than "=" or "!=". With this command for example,
we're left with PRN above 03 for GPS and below 10 (included) for Galileo

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P 'mask:gt:G08'
    -P 'mask:leq:E10'
```

## GNSS target

A comma separated list of Constellation is supported.  
For example, with the following, we're left with data from Glonass and GPS  

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P 'mask:neq:bds'
    -P 'mask:GPS,GLO'
```

## Carrier signals

A comma separated list of Carrier signals is supported.  
For example, with the following, we're only left with observations against L1 and L5

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P 'mask:l1,l5'
```

Carrier signals are one of the exceptions that support more than`eq` and `neq` operands.  
For example, with the following we retain L2, L5 signals and L1 is excluded:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P 'mask:gt:l1'
```

## Observables

A comma separated list of Observables is supported.  
For example, with the following, we're only left with phase observations,
against L1 and L2 carriers 

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P 'mask:L1C,l2c'
```

## Navigation frames

A comma separated list of Navigation Frames is supported.  
For example, with the following, we're only left with Ephemerides

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P 'mask:eph'
```

## Navigation Messages

A comma separated list of Navigation Messages is supported.  
For example, with the following, we're only left with legacy messages

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P 'mask:lnav'
```
 
Navigation Messages is one of those exceptions that support more that "=" (`eq`) or "!=" (`neq`) operands.  

With the following mask, we retain only modern navigation messages

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000.crx.gz \
    -P 'mask:gt:lnav'
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

## Signal condition filters

Observation data might have an "LLI" flag attached to them.
It is possible to apply an And() mask on this field. In this case,
all epochs that did not come with an LLI flag get also dropped out.

In this example, we focus on epochs where a `Loss of Lock` event happened

```shell
rinex-cli --pretty --lli-mask 1 --sv R01 \ 
          -f test_resources/OBS/V2/zegv0010.21o
```

SSI field is another data field that might come with an observation
and it gives the estimated receiver power / SNR at the sampling instant.

It is possible to filter data on minimum signal strength, which
is equivalent to a data "quality" filter

With the following command, we only retain data with SSI >= 5
that means at least 30 dB SNR. 

```shell
rinex-cli --pretty -f test_resources/OBS/V2/zegv0010.21o \
        --ssi-filter 5 --sv-filter R01
```
