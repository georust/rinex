Merge
=====

This tool allows Merging a secondary RINEX into a primary RINEX (`--fp`).  
The results is the combination of both, where
epochs and data were correctly associated.

When merging:

* both RINEX must be the same kind
* primary header section is always prefered
* if secondary header section comes with additionnal attributes,
they naturally get combined.

`-m [RINEX]` or `--merge [RINEX]` is used to trigger this operation

```bash
rinex-cli \
  --fp test_resources/OBS/V3/zegv0010.21o \
  -m test_resources/OBS/V2/delf0010.21o
```

This generates a new file, named "merged.rnx".

Like in other file generation scenarios, 
one can control the file to be generated, with `--output [RINEX]`

[RINEX] does not have to follow naming conventions

```bash
rinex-cli \
  --fp test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx \
  -m test_resources/NAV/V3/BRDC00GOP_R_20210010000_01D_MN.rnx.gz \
  -o merged_nav.rnx
```

Refer to the file production paragraph, to learn how to fully operate
this mode.
