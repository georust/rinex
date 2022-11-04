Split
=====

Splits a record at a desired epoch.

If User provides an `epoch`, the tool will try to locate the
given timestamp and perform `split()` at this date & time.

Two description format are supported, for the user to describe a sampling
timestamp:

* "YYYY-MM-DD HH:MM:SS" : Datetime description and EpochFlag::Ok is assumed
* "YYYY-MM-DD HH:MM:SS X" : Datetime description where X describes the EpochFlag integer value.
Refer to RINEX standards for supported Epoch flag values 

The tool identifies matching timestamp by comparing the datetime field AND
the flag field. They both must match.

Example :

```bash
# Split a previously merged record
cargo run -f /tmp/merged.rnx --split \
    --output /tmp/file1.rnx,/tmp/file2.rnx

# Split a record at specified timestamp,
# don't forget the \" encapsulation \" ;)
cargo run -f /tmp/data.rnx --split "2022-06-03 16:00:00" \
    --output /tmp/file1.rnx,/tmp/file2.rnx

# Split a record at specified timestamp with precise Power Failure event
# don't forget the \" encapsulation \" ;)
cargo run -f /tmp/data.rnx --split "2022-06-03 16:00:00 1" \
    --output /tmp/file1.rnx,/tmp/file2.rnx
```
