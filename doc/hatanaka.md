# Hatanaka

This section describes the `hatanaka` module,   
that comprises structures and methods to perform RINEX compression  
and decompression.

It is named after Yuki Hatanaka who created this compression algorithm.

## `CRINEX` decompression

To quickly decompress a CRINEX file into a RINEX,   
you should use this [this command line tool](https://github.com/gwbres/hatanaka),
based off this library.

Only Observation Data is to be compressed, therefore we are interested in decompression
such records.  
Thanks to this section of the library, the general `Rinex` parser is capable of
decompressing a `CRINEX` directly and expose its data to the user.

## Compression performances

Are reported by Y. Hatanaka, best compression performances
are obtained for a compression order of 4.

## Kernel

The `Kernel` structure is capable of applying the `Hatanaka` algorithm,
on both numerical data and text data.  
It is not limited to an M=5 maximal compression order,   
but M is fixed at runtime, for efficienty purposes.
