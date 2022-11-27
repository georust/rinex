## RINEX filtering

Two kinds of filtering operations:

* conservative/focus with `--retain`
* exclusive/filter out with `--filter`

For example this command will only retain data from specific vehicules

```bash
rinex-cli --retain-sv R03,G10 --fp test_resources/OBS/V2/rovn0010.21o
``` 

Teqc like GNSS filters are known. For example, `-R` and `-S`
will filter out Glonass and SBAS constellations

```bash
rinex-cli -R -S --fp test_resources/OBS/V2/rovn0010.21o
``` 

Filtering and resampling operations can be stacked together, for efficient
record focus:

```bash
rinex-cli \
    --fp rovn0010.21o \
    --retain-sv G10 \
    --decim-interval 00:05:00
```

Filtering and resampling operations are preprocessing operations.  
That means they apply to all modes this tool supports:

* RINEX identification
* Record analysis
* RINEX processing
* ...

## Observations / Data keys filter

Observation and Meteo RINEX are sorted by Observation. 
The user can focus on data of interest with `--retain-obs`,
to which an array of Observation Codes can be passed.

For example, focus on Phase and Pseudo Range observations
in old RINEX format:

```bash
rinex-cli -f zegv0010.21o  --retain-obs C1,L1
```

A quick RINEX identification helps identify which "observables" exist:

```bash
rinex-cli \
    -f zegv0010.21o --observables --pretty
```

In case of `--fp` is Navigation RINEX, we have the `--orbits` command to perform similar
data identification

```bash
rinex-cli \
    -f CBW100NLD_R_20210010000_01D_MN.rnx \
    --orbits --pretty
```

Navigation RINEX are sorted by orbits. The `--retain-orb` is the `--retain-obs` equivalent:

```bash
rinex-cli \
    --retain-orb cus,omega0 \
    -f CBW100NLD_R_20210010000_01D_MN.rnx
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
