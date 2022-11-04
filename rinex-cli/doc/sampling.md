## RINEX resampling

Resampling operations are designed to reduced the record data quantity.
They do not impact the file Header section. 
They also only apply to RINEX files that are indexed by `Epochs`,
refer to the [main table](https://github.com/gwbres/rinex/blob/main/README/#supported-rinex-types)

Decimation (down sampling) can be performed in several ways

- either by an integer ratio
- or by a minimum epoch interval ("sampling interval") to follow

It is also possible to apply a "time window" to the RINEX
record, all epochs that do not lie within the a < e(k) < b predicate
will get filtered out.

Example: decimate record by 2.
From original epochs a,b,c,d, we're only left with a,c
```bash
rinex-cli -f test_resources/OBS/V2/zegv0010.21o \
    --resample-ratio 2
```

Example: decimate to match a minimal sampling period.
When describing a sampling period (chrono::Duration),
we expect an %HH:%MM:%SS format, which can exceed 24 hours
```bash
# print original epochs
rinex-cli -f test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz \
    --epochs
# Fit minimal sampling period to 1 day
rinex-cli -f test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz \
    --epochs \
        --resample-interval 24:00:00
```

It is also possible to restring epochs to a given interval (or "time window").
When describing a time window, we can either describe a Date interval (%Y-%m-%d)
or a DateTime interval (%Y-%m%d %HH:%MM:%SS).
Since this descriptor involves whitespaces, it must be quoted

```bash
# print original epochs
rinex-cli -f test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz \
    --epochs
# restrict to last day contained in record
rinex-cli -f test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz \
    --epochs \
        --time-window "2022-06-09 2022-06-11" 
 
# grab some of the last epochs of first day and entire final days
rinex-cli -f test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz \
    --epochs \
        --time-window "2022-06-08 11:00:00 2022-06-11 12:00:00" 
```
