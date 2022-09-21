Hatanaka
========

This section describes the `hatanaka` module,   
that comprises structures and methods to perform RINEX compression  
and decompression.

It is named after Yuki Hatanaka who created this compression algorithm.

According to Y. Hatanakas specifications,
best compression performances
are obtained for a compression order of 4.

The official `CRNX2RNX` and `RXN2CRNX` tools have m=5 builtin.

As opposed to these tools, this library allows the user to specify the compression
order. On the other hand, we are currently limited to m=7 maximal compression order.

## Data compression

We support Text and data compression.

Equations:

## Data decompression

We support Text and data decompression.

Equations:
