RINEX Preprocessing
=================== 
  
It is important to master the preprocessing filter designer to operate this tool efficiently.

Several algorithms are known:  

* [Data masking](#masking-operations)
* [Data scaling](#scaling-filters)
* [Data decimation](#data-decimation)
* [Data smoothing](#data-smoothing)
* [Data interpolation](#interpolation-filters)

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

A whitespace separates two preprocessing operations.
Therefore, if a filter operation involves a whitespace, it requires to be wrapped
in between inverted commas. Most common example is [Time windowing](time-windowing) operation.

In any case, invalid descriptors will not crash the app but only generate an error trace.

All supported preprocessing operations can either apply to the entire data set
or only to a specified data subset, see [this paragraph](#advanced-data-subsets) for more information.

## Masking operations

Use Mask filters to focus on data you are interested in, or get rid of entire data subsets.

A mask filter is one operand and one data subset to match.

### Operands

List of supported Operands:

* Lower Than (<) 
* Lower Than or Equals (<=) 
* Greater Than (>)
* Greater Than or Equals (>=)
* Equals (=)
* Ineq (!=)

When the operand is omitted in the description : "equality" is implied.

## GNSS constellation masking

Retain Galileo and Glonass : 

```bash
rinex-cli -P GAL,GLO [...]
```

Drop Galileo: 

```bash
rinex-cli -P !=GAL [...]
```

teqc like GNSS filters are also supported :

- `-G` : remove GPS vehicles
- `-R` : remove Glonass vehicles
- `-E` : remove Galileo vehicles
- `-C` : remove BDS vehicles
- `-J` : remove QZSS vehicles
- `-I` : remove IRNSS vehicles
- `-S` : remove SBAS vehicles

## Satellite vehicle masking

Retain G08 and R03 only :

```bash
rinex-cli -P G08,R03
```

Retain all GPS vehicles but G15 and G16

```bash
rinex-cli -P GPS !=G15 !=G16
```

Use other operands to retain specific PRN. Retain PRN above 08 (included):

```bash
rinex-cli -P GPS >=G08
```

Decimate G15 record quantity by 2, leaver others untouched :

```bash
rinex-cli -P GPS decim:2:G15
```

## Time windowing

Select a single epoch : because _Equals_ operand is implied
when operand is omitted in the description:

```bash
rinex-cli -P ">2020-06-12T08:00:00 UTC"
```

Select a time frame above given epoch.
Notice the \" due to whitespace requirement.  
Any operand works here:

```bash
rinex-cli -P >="2020-06-25T04:00:00 UTC" 
```

Define a time window : here we're left with 8 hours of data :

```bash
rinex-cli \
    -P ">2020-06-12T08:00:00 UTC" "<=2020-06-25T16:00:00 UTC"
```

## Record decimation
  
Use record decimation to reduce data quantity.

Decimate data quantity by 2 :

```bash
rinex-cli -P decim:2
```

Decimate all Galileo data by 50 %:

```bash
rinex-cli -P decim:2:gal
```

Decimate L1C observations by 4 but L2C observations by 2: 

```bash
rinex-cli -P decim:2 decim:2:l1c
```

Because preprocessing operations are executed 
one after the other.

## Observation masking

Retain L1C observations only : 

```bash
rinex-cli -P l1c
```

Retain all observations but L1C (case tolerant):

```bash
rinex-cli -P !=L1C
```

Retain humidity rate and rain increment observations only : 

```bash
rinex-cli -P hr,ri
```

Any valid observable symbol works here: "L1C", "L2P" for example
for OBS RINEX, or "TD", "HR", "WS".. for METEO RINEX.
This is also case tolerant.

## Orbit fields masking

Retain specific orbit fields in NAV RINEX, with any
valid Orbit field identifier:

```bash
rinex-cli -P iode,crs,cus
```

See the rinex/db/nav.json file to retrieve the list of orbit fields identifier.

Decimate all this record by 2, but cus and crs by 2 :

```bash
rinex-cli -P decim:2 decim:2:cus,crs
```

## Navigation frames

Retain ephemeris frames only when analyzing NAV RINEX:

```bash
rinex-cli -P eph
```

Retain ephemeris and ionospheric models with :

```bash
rinex-cli -P eph,ion
```

Any valid NAV frame works here.

Reduce the quantity of ephemeris frames by 2, leave others as is:

```bash
rinex-cli -P decim:2:eph
```

## Navigation Messages

Any valid navigation message symbolization is known. For example,
retain only legacy NAV with :

```bash
rinex-cli -P lnav
```
 
## Elevation mask

Elevation masking is done with an "e" prefix.
They currently only apply to NAV RINEX.

Retain vehicles above 10° angle with :

```bash
rinex-cli -P "e> 10.0"
```

Combine two elevation masks to create elevation range conditions:

```bash
rinex-cli -P "e>10.0" "e<=45"
```

## Azimuth mask

Same thing applies to azimuth angle, but with an "a" prefix.

## Data smoothing

Use data smoothing algorithms to smooth data sets.

### Moving average

Invoke a moving average filter with "mov" and a valid duration description:

Comparison between raw and smoothed temperature data over the course of an entire day:

```bash
rinex-cli \
	-P td \
	--fp test_resources/METEO/V1/TODO
rinex-cli \
	-P td mov:30 min \
	--fp test_resources/METEO/V1/TODO
```

Apply a 1min moving average over the signal strength of L1 :

```bash
rinex-cli \
	-P mov:1 min:s1c \
	--fp test_resources/METEO/V1/TODO
```

### Hatch filter

The Hatch filter is a smoothing algorithm that applies specifically
to pseudo range observations.

TODO

```bash
rinex-cli \
	-P hatch:l1c \
	--fp test_resources/METEO/V1/TODO
```

## Data scaling

Several data scaling or rescaling methods exist.

### Static offset

Offset all observations by +10 (raw value)
```bash
rinex-cli \
	-P hatch:l1c \
	--fp test_resources/METEO/V1/TODO
```

Offset all temperature observations by +25.5 : 
```bash
rinex-cli \
	-P hatch:l1c \
	--fp test_resources/METEO/V1/TODO
```

Offset all L1C phase observations by -3.14 :
```bash
rinex-cli \
	-P hatch:l1c \
	--fp test_resources/METEO/V1/TODO
```

### Scaling

Apply an (a, b) scaling so every observation on epoch _k becomes
y_k = x_k * a + b :

```bash
rinex-cli \
	-P scaling:10.0,5.0 \
	--fp test_resources/TODO
```

### Remapping

Remapping, for the lack of a better term, 
squeezes the dataset or subset into N groups.

For example, we currently use this a lot when processing IONEX data
because it is heavy to plot and render such data. In this example,
we take all the TEC values and squeeze them into 4 different groups (Very high, high, low, very low)
and you're only left with 4 different TEC values to represent :


```bash
rinex-cli \
	-P remap:4 \
	--fp test_resources/TODO
```
