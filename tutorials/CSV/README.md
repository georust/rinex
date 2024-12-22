CSV
===

`rinex-cli` offers multiple options to convert either RINex or SP3,
basically all supported formats, to CSV, which might prove convenient
to export data to external tools.

To export data to CSV you have several options
- the `--filegen` option, which basically means "synthesize data from input",
will let you select CSV instead of RINex/SP3
- basically all file operations support the `--csv` option, for example
you can directly convert the TBIN batch or the differentiated observations
to CSV (see specific examples)

Examples:
  - obs.sh: extract observations (signals) to csv,
  for example to plot with Python
  - meteo.sh: same principle, different RINex
  - spp.sh: SPP compliant context extraction
  - ppp.sh: PPP compliant context extraction
  - diff.sh: RINEX(A) - RINEX(B) differential ops exported to CSV
  - tbin.sh: design a 1Hr batch and export to CSV
