File splitting
==============

The `split` mode of `rinex-cli` allows splitting two files at a specific
date and time of their time frame. The datetime of the splitting operation
is passed right after `split` and needs to describe a valid epoch.
This operation may apply to any temporal fileset.

Once A as been split into B and C, we can either dump B and C as RINex, SP3
or CSV.

Examples:
- `obs.sh` split signal observations at noon
- `sp3.sh` split SP3 at noon
- `obs-csv.sh` split signal observations at noon and export to CSV directly
- `meteo.sh` split meteo sensors observations at noon
- `ionex.sh` split TEC maps at noon
