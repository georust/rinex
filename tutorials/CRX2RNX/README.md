CRX2RNX
=======

These examples demonstrate the CRINEX decompression and RINEX formatting capabilities
of this toolkit.

The CRINEX decompressor uses the same internal parameters as the historical `crx2rnx` program,
therefore is totally compatible with those files. Indeed, all the files we host in this repo
were compressed using the historical program. By default, our toolkit uses those parameters.

Our toolkit does not limit itself to a (de-)compression level of 3 like the historical tool.
You have complete control over the maximal compression order.
A compression order of 5 is said to be the optimal (following Y. Hatanaka's original paper).
But keep in mind that CRINEX is not a lossless compression when it comes to signal observations: 
the higher the compression order, the larger the error.

