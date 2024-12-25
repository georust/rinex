Merge
=====

File merging is a convenient operation when producing data.  
It is one of the file operations avaiable in the `rinex-cli` toolbox.

When merging two files, A & B must have the same format, you cannot
merge a meteo RINex into observation RINex for examples. 
You can obviously merge a CRINex into a readable observatio RINex:
because it is the same underlying format.

Note that merging data sets that have two different origins is most likely
an invalid operation. Yet this toolbox allows such operation,
which will allow you to perform weird or exotic operations. It is up to you
to know what you're doing.

The C header will reflect the merging operation, especially

- the revision of the software used to perform the operation
- the date and time of the merging operation
- conversion to mixed constellation if need be
- conversion to mixed signals and modulations if need be
- scaling conversions, if need be

Examples :

- obs.sh: merge two observations RINex together
- crinex.sh: merge non readable compressed RINex into either
readable to compressed RINex
- obs-csv.sh: merge and stream to CSV directly
- meteo.sh: merge meteo observations together.
- ionex.sh: merge two IONex together, creating a 48h time frame in this example 
