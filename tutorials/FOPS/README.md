File Operations
===============

All our tools permit `File Operations` per say.

The most basic tools like `crx2rnx` will generate one output per input.  
`rinex-cli` offers more complex operations, that are usually required by
GNSS post processing pipelines.

- [CRX2NRX](./CRX2RNX) describes how to operate the `rnx2crx` tool.
It also describes `rinex-cli` options that are related to the CRINEx compression algorithm.
- [RNX2CRX](./RNX2CRX) describes how to operate our `crx2rnx` tool,
and `rinex-cli` options that are related to the CRINEx decompresion algorithm.
- [DIFF](./DIFF) illustrates the `diff` opmode of `rinex-cli`, which allows
observations differentiation
- [SPLIT](./SPLIT) illustrates the `split` opmode of `rinex-cli`, which allows
to split any temporal input product `A` into two `B` and `C` products.
- [FILEGEN](./FILGEN) illustrates the `filegen` opmode of `rinex-cli`, which is
our option to patch RINEx or SP3 products into identical yet reworked products.
- [MERGE](./MERGE) describes the `merge` opmode of `rinex-cli`, which allows
to combine two similar `A` and `B` input products into one.
- [TBIN](./TBIN) describes the `tbin` (time binning) opmode of `rinex-cli`, which
allows to split any temporal product into a batch 
