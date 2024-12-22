Time Binning
============

`tbin` (for timing binning) allows creating a fileset where each individual file
has the same time frame duration. It applies to any temporal RINex or SP3.

`tbin` supports all file operations options, that includes direct CSV output
(instead of RINex/SP3), or gzip compression for example.

Examples:
- `esbjrg-tbin6x4.sh` split CRINex into 4 either
compressed or readable RINex, of 6 hour duration each
