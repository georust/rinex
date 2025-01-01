Filter Designer
===============

The toolbox integrates a filter designer which is summoned by `-P` (for Preprocessing).  
One needs to understand that the preprocessor will apply to the entire fileset.

The preprocessor is a simple option to focus on particular signals or constellations.
For example, let's say you are interested in analyzing `GPS` data but not any other.

Historical filters
==================

`rinex-cli` supports a few options that `teqc` has:

- `-G` removes GPS (equivalent to `-P !=GPS`)
- `-C` removes BDS
- `-E` removes Galileo
- `-R` removes Glonnass
- `-J` removes QZSS
- `-S` removes all SBAS vehicles

Filter specifications
=====================

Design a filter with `-P`.  For example `-P GPS` is a valid filter.  

The preprocessor supports 6 operands:

* `<` Lower Than 
* `<=` Lower Than or Equals 
* `>` Greater Than
* `>=` Greater Than or Equals
* `=` Equality
* `!=` Inequality

When the operand is ommited, like in `-P GPS`, it is the `Equality` 
operand that is implied. One consequence of that is, that if you're interested in discarding data, you most likely will need an operand.

For example, `-P !=Gal` will preserve everything but Galileo.

Lower than and Greater than only apply to time frames
or SV. For example `-P >Gal` does not make sense.
That means, only `Equality` or `Inequality` may apply to some cases like Constellations.

You can then stack many filter operations to create a complex combination. For this you have two options:

- use `-P` as many times as you need.
All operations will be serialized and applied one after the other.
They are applied in the specified order.
On example would be `-P "!=Gal"  -P "!=GPS"` to retain anything but GPS or Galileo.
- use `-P` and `;` to describe many operations at once, `-P "!=Gal;!=GPS"`

The Filter specifications are case insensitive.   
Some items are smarter than others, for example, we support
RINEx codes to describe signals and modulations: `-P C1` or `-P C1C`.  We support the RINEX code and the constellation name for Constellations. For example `-P GAL` or `-P Galileo` both work. The previous `-P Gal` example also works, because the filter designer is case insensitive.

The filter designer is tolerant to missing whitespaces between the operand and the target. For example:

rinex-cli -P "Gal;<2024-08-24T10:00:00 UTC" [...]

The filter designer supports many items:

- [Constellations](#constellations-filter)
- [SV](#sv-filter)
- [DateTime](#datetime-filter)
- [Signals and modulations](#signals-and-modulations)
- [Decimation (subsampling)](#decimation-(subsampling))

Constellations filter
=====================

Retain specific constellations: for example, retain GPS only with:

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
    -P "!=GLO"
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
SBAS and geostationnary
=======================

If you have worked with SBAS and geostationnary vehicles with our toolbox, you have most likely noticed that they are plainly named in our ecosystem.

You can specifically name any SBAS constellation in your filter specifiations. For example, `-P !=EGNOS` will preserve any satellite vehicle, whether is is LEO, MEO or GEO as long as it is not part of `EGNOS`.

SV filter
=========

Retain the vehicles you're interested in, using a CSV list of vehicles.

Example: retain 08, 09, 10 from GPS :

```bash
rinex-cli \
    -P "G08,G09,G10" \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
```

Example: retain any vehicles but 08, 09, 10 from GPS :

```bash
rinex-cli \
    -P "!=G08,G09,G10" \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
```

## SV filter with operand

The filter operand does work when specifiying SV identities. 
In this case, it is used to filter (in or out) the PRN you want for a given
Constellation.

Example (1): exclude GPS vehicles below 08 (excluded) 

```bash
rinex-cli \
    -P ">G08" \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
```

Example (2): retain PRN above 08 for GPS, and below 10 (included) for GAL :

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P ">G08;<=E10"
```

Signals and modulations
=======================

In `RINEx` (Receiver Independent Exchange format),
signals, physics and modulations are represented by `Observables`.

Observables are valid filter targets. Any valid RINEX code is accepted.

For example:

- `-P L1` will preserve any L1 phase range observation (old RINEX code), because equality is implied.
- `-P !=L1` will preserve anything but L1
- `-P L1C` will specifically retain L1 civilian phase range observations (modern RINEX code fully describe the modulation).
- Greater than and lower than do not apply to Observations because they do not make sense in that situation: only equality and unequality are supported.
- Observables may describe more than signals observations. As previously stated any valid observable description is supported. This may also serve Meteo or DORIS observations. For example: `-P TD` will only retain temperature measurements

Datetime filter
===============

Epoch filter can be used to perform time binning, i.e redefining
the time frame of your data.

Any valid Hifitime::Epoch string description is supported.  

Since equality is implied, if you do this you're left with a single Epoch
in your context:

```bash
rinex-cli \
    -P "2020-06-25T04:00:00 UTC" \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
```

Use the inequality operand to retain anything but that very Epoch

```bash
rinex-cli \
    -P "!=2020-06-25T04:00:00 UTC" \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
```

## Time window

Stack two Epoch filters to define a time window :

```bash
rinex-cli \
    -P ">2020-06-12T08:00:00 UTC; <=2020-06-25T16:00:00 UTC" \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
```

Decimation (subsampling)
========================

The preprocessor (`-P`) supports different resampling algorithms.

Record decimation (subsampling) is specified with the `decim:` prefix.

## Decimate to new sampling interval

In this example, the original sampling period is reduced to 10 minutes.  
You can either proceed with record analysis (no option), here we request output product synthesis,
with `filegen`:

```bash
rinex-cli \
    --fp test_resources/MET/V3/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz \
    -P "decim:10 min" \
    --filegen
```

Like any preprocessing, you can stack it to other options, to create complex conditions:

```bash
rinex-cli \
    --fp test_resources/MET/V3/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz \
    -P "decim:10 min" \
    -P ">2020-06-25T08:00:00 UTC" \
    -P "<=2020-06-25T10:00:00 UTC"
```

Like any preprocessing operations, decimation may apply to any supported opmodes, for example `ppp`.

## Modulo decimation

When passing a simple integer, we specify a decimation factor.

In this example, the 30s sampling period is reduced to 1 min:

```bash
./target/release/rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -P decim:2 \
    --filegen
```
