Quality Check (QC)
==================

RINEX quality check is performed on Observation Data by providing
such RINEX with `--fp` and requesting `--qc`.

QC is a well known procedure in data preprocessing and this tool
tries its best at emulating what `teqc` is capable of.   
There are a few differences people who are very familiar with `teqc` must
take into account when using our command line interface.

Teqc / Cli differences
======================

Unlike teqc we are not limited to RINEX V2, V3 and V4 Observations
are fully supported.

Unlike teqc we expect the user to provide the file context
himself. There is not such thing as auto search of a possible Navigation
Ephemeris for instance. This is provided with `--nav` for instance.

Like most UNAVCO tools, we will generate products in a dedicated folder.  
The current behavior is to use the 
[product](https://github.com/gwbres/rinex/tree/rinex-cli/product)
folder, that came with this tool, and we generate in this location,
a folder that bears the `-fp` name. That means, you can
run several analysis (against different `-fp`) and we maintain the results.

Unlike teqc we will probably not limit the QC operation
to Observation data, although that is not the case to this day.

The following differences are important to understand or, at least be aware of,
in case you intend to pass the reports we generate to a "teqc" report parser.

Unlike teqc we do not limit ourselves to the analysis of
GPS and Glonass constellations.
Historically, teqc would analyze both (if requested),
in two seperate sections of the summary report.
We decided we would split our analysis into several summaries: one per constellation,
to shorten the file length.

Unlike teqc, we have no means to detect epoch duplicates
and duplicated SV accross epochs. This information is not
contained in the report we generate.

Unlike teqc, we do not limit ourselves to L1/L2 analysis.  
This applies for instance to MPx (Code Multipath biases),
averaged received signal strength estimates, etc.. 

Differences and retro compatibility
===================================

- grab the report summary for the constellation you're interested in
- The epoch timestamp are reported in a different format:
YYYY-MM-DD-HH:MM:SS{TS} where TS is the timescale under use, for example "UTC".
This mainly impact "Time of start and end" windows reporting.

- Tick rate, or sample rate is reported in a more convenient fashion.
Mainly, we do not limit ourselves to a 1 hour granularity.
Our theoretical limitation is 1 ns, 100 ns in practice.
This only impacts the "Time line window length" report.

- We don't report "infinity" but 0 when receiver was not reset during
sampling. Also, this duration would not be limited to minutes in case
such an event duration was to be estimated, as previously stated.

- Receiver clock drift reporting is no longer limited to milliseconds per hour.
So unit must now be parsed: you can't assume we will always report ms/hr.
This allows accurate drift reporting and estimates, below 1us/hr,
which was the previous limitation.

- analysis against L>2 carrier may exist.
For example, you may find "Mean S5" for averaged received L5 signal strength,
or MP15 for 1/5 code multiath bias
- "Obs w/ SV duplication" field cannot be formed, it no longer exists
- "Epochs repeated" cannot be formed, it no longer exists

Other than that, we strictly follow the teqc format.

Basic QC
========

Basic QC requires a navigation context to be provided,
[otherwise](https://github.com/gwbres/rinex/blob/main/rinex-cli/qc.md#minialist-qc),
most of the relevant calculations cannot be performed.

When requested, QC generates a summary report in the product/ subfolder.  
We form one report per encountered constellation, and we do not limit ourselves
to GPS and Glonass. Obviously, if the provided data is stripped to a single
constellation, you get a single report.

Example of a standard qc:

```bash
rinex-cli --qc \
   --fp test_resources/CRNX/V3/KMS300DNK_R_20221591000_01H_30S_MO.crx
```

Summary report
==============

<img align="center" width="400" src="https://github.com/gwbres/rinex/blob/main/doc/ascii-plot.png">

Quiet QC
========

Quiet qc is requested by adding the `-q` flag, not to be mistaken with `--qc`. 
`-q` basically turns off all terminal output this tool might generate.  
In the case of QC, that means you get the summary report in the file, but not
in stdout.

Minimalist QC
=============

When `--nav` is not provided, several information cannot be determined.  
Mainly elevation, satellite health and satellite attitude related informations.  
The ascii plot is therefore reduced in this use case.

Example of a minimalist qc

```bash
rinex-cli --qc \
   --fp test_resources/CRNX/V3/KMS300DNK_R_20221591000_01H_30S_MO.crx
```

