Tutorials
=========

The tutorials serie will try to illustrate all options offered by the toolbox.

Prior running our examples, you are expected 
[to read how the two Wiki pages](https://github.com/georust/wiki)

## Geodetic reports

The [Qc (Quality Control)](./QC) reports and most broadly, _geodetic report_ 
is heavily documented and probably the most important feature
of our toolbox: 

## File operations

The toolbox can perform several file operations.
File operations refer to operations where we're either

1. interested in reworking on patch an input product
2. always synthesizing at least one output product.
Whether it is a RINEx, and SP3 or other format depends on the context.

Follow [this section](./FOPS) if you're interested in such operations.

## Navigation

Post processed navigation and surveying is depicted [in the related section](./NAV).

It solely relies on `rinex-cli` to this day. It depicts static and other contexts
of navigation.

## CGGTTS

The [CGGTTS](./CGGTTS) section focuses on the post processed _timing oriented_ navigation solution.

## Other

When operating the toolbox, it might be very useful, sometimes vital, to use the debug traces
and understand what they mean. [This page](./Logs.md) teaches how to unlock the debug traces
and their meaning.
