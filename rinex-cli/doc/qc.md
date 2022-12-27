Quality Check (QC)
==================

RINEX quality check is a special mode of this tool, activated with the `--qc` option.

QC is first developed for Observation files analysis, but this tool  
will accept other RINEX files, for which it will compute basic statistical anlysis. 

QC is a well known procedure in data preprocessing. 
There are a few differences people who are very familiar with `teqc` must
take into account when using our command line interface.

## Differences with `teqc`

Unlike teqc we are not limited to RINEX V2, V3 and V4 Observations
are fully supported.

Unlike teqc we expect the user to provide the file context
himself. There is not such thing as auto determining possible Navigation context
in predefined folders. This tool expects all files to be provided with an argument
  
1. `--fp [FILE]` for the Observation file
2. `--nav [FILE1] [FILE2]..` for secondary Navigation files  

Like most UNAVCO tools, we will generate products in a dedicated folder.  

The current behavior is to use the 
[product](https://github.com/gwbres/rinex/tree/rinex-cli/product)
folder to generate QC reports.

Unlike teqc, we do not support BINEX nor SP3 input data/files as of today.

Unlike teqc we do not limit ourselves to the analysis of
GPS and Glonass constellations.

Unlike teqc, we have no means to detect epoch duplicates
and duplicated SV accross epochs. This information is therefore missing.

Unlike teqc, we do not limit ourselves to L1/L2 analysis.  
This applies for instance to MPx (Code Multipath biases),
averaged received signal strength estimates, etc.. 

Unlike teqc, this tool allows accurate time description, down to 1 ns precision.  
For example, this would apply to

* the receiver clock drift analysis not being limited to 1ms/hour  
* precise control of averaging windows, etc..

## QC specific command line options

* `--qc-separate`: use this option to generate the QC report in its own HTML report
* `--qc-only`: ensures the tool will only perform the QCs, other graphs and analysis are turned off
* `--qc-config`: pass a configuration file for QC reporting and calculations management (see down below) 

## Basic QC (No NAV)

When `--qc` is activated and `--fp` is an observation file
basically we can only study the provided signals and observations.

Preprocessing still apply, therefore all analysis are to be performed
on the data that is still left out:

```bash
rinex-cli \
    --qc \
   --fp test_resources/CRNX/V3/KMS300DNK_R_20221591000_01H_30S_MO.crx
```

(Enable and) Separate the report:

```bash
rinex-cli \
    --qc-separate \
   --fp test_resources/CRNX/V3/KMS300DNK_R_20221591000_01H_30S_MO.crx
```

Run this configuration for a very basic QC:

```bash
rinex-cli \
    -F mask:gps,glo \
    --qc-separate \
    --qc-conf rinex-cli/config/basic.json \
   --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
```
  
basic.json specifies that we want to report on a constellation basis.  
If you compare this report to the previous one

1. you get one table per constellation. We kept GPS and Glonass only, therefore you have two tables.
2. therefore all statistical analysis are independent, for each constellation.  
3.  A "25%" statistical window is specified. "window" accepts either a Duration  
or a percentage of the file. Here we use the latter, and 25% means we will have 4 statistical  
analysis performed over the course of 8 hours, based on this file.
4. you also have less information, that is because basic.json has turned off  
several calculations (see down below)

Try this configuration now:

```bash
rinex-cli \
    -F mask:gps,glo \
    --qc-separate \
    --qc-conf rinex-cli/config/basic_manual_gap.json \
   --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
```

A "10%" statistical slot is specified, you get shorted statistical analysis spanning  
around 2 hours in this context.  
Also "manual_gap" is specified and set to "10 seconds". That means that a duration  
of 10 seconds is now considered as an abnormal gap, in the data gap analysis.  
In default configuration, there is no manual gap. That means an abnormal gap  
is any abnormal duration above the dominant epoch interval (sample rate).

When no configuration files are given, the default configuration is used

```json
TODO
```

## Advanced configurations

Now let's move on to more "advanced" configuration, in which basically all   
calculations are active and customized

```bash
rinex-cli \
    -F mask:gps,glo \
    --qc-separate \
    --qc-conf rinex-cli/config/basic_manual_gap.json \
   --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz
```

## Basic QC (with NAV)

Let's go back to our basic demo and provide Navigation context:

```bash
rinex-cli \
    -F mask:gps,glo \
    --qc-separate \
    --qc-conf rinex-cli/config/basic.json \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --nav test_resources/OBS/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz
```
  
Navigation context is fully taken into account in advanced calculations

```bash
rinex-cli \
    -F mask:gps,glo \
    --qc-separate \
    --qc-conf rinex-cli/config/advanced_study.json \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --nav test_resources/OBS/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz
``

