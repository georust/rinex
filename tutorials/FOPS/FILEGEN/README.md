File generation
===============

`filegen` is our option to specify you would like to generate RINEx or SP3 products,
instead of the analysis report. It is particularly useful to take advantage of our preprocessor (`-P`)
to generate your own "customized" data.

A combintion of several runs, for example an initial `merge`, followed by `filegen` with
a custom filter can create complex processing pipelines.

`filegen` is compatible with `--csv`, like any other file operation, to dump to CSV instead
of RINEx. The `rnx2crx` and `crx2rnx` options may also apply, depending on the output you want to synthesis.
When synthesizing RINEx, the file naming conventions may be applied and the customization options are still relevant
in this opmode.
