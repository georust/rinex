Split
=====

This tool allows splitting a RINEX file (`--fp`) into two.

This operation is triggered by `-s [DATETIME]` where DATETIME
is a correct "YYYY-MM-DD" or "YYYY-MM-DD HH:MM:SS" datetime description.

Example: split `ESBC00DNK_R_2020̀_MN  at midday (12 hours into the file):

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -s "2020-06-25 12:00:00" 
```

This will generate two files 
* "2020-06-25-000000.rnx": that contains the first half of that day
* "2020-06-26-120000.rnx": that contains the remaining of the data

HH:MM:SS can be omitted in the timestamp description,
in this case we consider midnight 00:00:00.

For example, split `ESBC00DNK_R_2020_MN` contains the last
2 hours of day 2020-06-24 and the following day was entirely sampled.
The following command would split both days:

```bash
rinex-cli \
    --fp test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -s "2020-06-25" 
```

This will generate "2020-06-24-220000.rnx" and "2020-06-25-000000.rnx".
