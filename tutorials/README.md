Scripts
=======

List of Shell scripts to operate our applications (no GUI available yet) and perform interesting tasks.

Getting started
===============

All scripts are intended to be executed at the base of this Git repository, see the following example.   
They also expect the released application to be built with all features (heaviest option):

```bash
# download the toolbox and dataset
git clone https://github.com/georust/rinex
cd rinex
# build it with release, for efficient experience
cargo build --release --all-features

# try one of the examples
./tutorials/GAL/mojdnk.sh
```

RINEX-Cli and other applications will generate logs but we do not store them in the following examples.  
Activate the application logs by activating the `RUST_LOG` environment variable.  
For example, this will make you see any trace

```bash
export RUST_LOG=trace
```

Surveying
=========

Field surveying (currently only static) aims at determining the position of
reference stations very precisely without a priori knowledge, so they can serve later on as reference stations for differential
positioning techniques.

Most scripts are sorted by constellation being used in the navigation/surveying or analysis process, either
single or a combination of constellations might be used, possibly GEO or SBAS service too.

`-brdc` emphasizes _real time_ surveying using radio messages, as opposed to _post-processed_ 
surveying, that exhibits higher accuracy.

`-qc-sum` scripts will only generate a summary report (shortened). The summary
(quicker) report is typically used along other opmodes (like `ppp`) when solely focused on post processing,
plain (lengthy) reports are prefered prior post processing, to adjust or verify parameters. 

CGGTTS solutions are special timing oriented solutions, to compare remote clocks to one another. 
We demonstrate the synthesis of CGGTTS solutions with our position surveys.

Other file operations like RINEX files management is also demonstrated.

- [48H](./48H) two RINEX datasets at once and more
  - 48h surveying (2 day course with double RINEX)
- [BeiDou](./BDS) constellation post-processing
  - Esbjerg and Mojn (DNK) stations surveying
- [BeiDou (GEO)](./BDS-GEO) stationnary vehicles applications
  - Esbjerg and Mojn (DNK) stations surveying
  - Navigation with GEO augmentation
- [BINEX](./BINEX) application
- [CGGTTS](./CGGTTS) oriented examples
  - CPP for CGGTTS synthesis
  - PPP for CGGTTS synthesis
  - CGGTTS and common view comparison
- [CONFIG](./CONFIG) are special preset to operate the toolbox
more precisely. It contains a fully commented example for each script.
- [CRX2RNX](./CRX2RNX) CRINEX decompression to readable RINEX (file I/O)
- [RNX2CRX](./RNX2CRX) RINEX compression to compact CRINEX (file I/O)
- [Galileo](./GAL) constellation post-processing
  - Esbjerg and Mojn (DNK) stations surveying
- [Galileo with SBAS augmentation](./GAL_SBAS)
- [Clock](./CLK) special RINEX applications
  - Precise Clock products
  - PPP Navi with Clock or SP3 only (example)
- [CSV](./CSV) output (to third party tools?)
  - RINEX to CSV extraction capabilites
  - SP3 to CSV extraction capabilities
- [File Synthesis](./FILEGEN) after pre-processing
  - RINEX pre-processing and format (File I/O)
  - SP3 pre-processing and format (File I/O)
- [GPS](./GPS) constellation post-processing
  - Esbjerg and Mojn (DNK) stations surveying
- [JMF: sampled by J.M. Friedt @ femto-st.fr](./JMF) french lab
  - 2024-092 Mobile phone observations (Paris/urban)
  - 2024-110 Nyalesund (NOR) 2024 glacier surveying (RTK compatible)
  - 2024-111 Nyalesund (NOR) 2024 glacier surveying (RTK compatible)
- [QC](./QC) (other)
  - other QC and analysis examples (without surveying)
- [OBS](./OBS) Simple Observation RINEX exploitation
- [SP3](./SP3) high precision orbit applications 
  - Examples with SP3 data only 
  - SP3 for PPP
- [RINEX(A) - RINEX(B)](./DIFF) simple yet efficient differential analysis
  - Esbjerg and Mojn (DNK): close range observations
- [Meteo](./METEO) special RINEX applications
  - Modern Meteo observations (QC)
  - Meteo for accurate Troposphere Modelling in PPP
- [Time binning](./TBIN) into a batch of files (File I/O)
  - Esbjerg (24h/DNK) station observations
  - SP3 batch generation
- [Time splitting](./SPLIT)
  - Esbjerg (24h/DNK) station observations
  - SP3 batch splitting
