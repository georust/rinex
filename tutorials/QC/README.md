QC tutorials
============

`QC` stands for Quality Control and is the broad default option of `rinex-cli`,
which will generate a geodetic report that summarizes everything the provided dataset contains.

When no options is passed to `rinex-cli`, we consider this is the selected mode
and the program will render the report in HTML format. 

- [Summary](./Summary) describes the most basic Qc report one can synthesize.
- [Observation](./Observation) describes the Qc reporting behavior when
Observation RINEx is present, and also available options in that scenario
- [Navigation](./Navigation) same thing for Navigation RINEx
- [SP3](./SP3) same thing when at least one SP3 file exist in the context
- [IONEX](./IONEX) describes the Qc reporting behavior and available options
when at least one IONEx file was provided to the context
- [Clock](./Clock) same thing for special Clock RINEx files
