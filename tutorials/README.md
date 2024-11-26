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

- [Config](./config) are special preset that some tutorials may use
- [CRX2RNX](./CRX2RNX) CRINEX decompression to readable RINEX (file in/file out)
- [GPS](./GPS):
  - Esbjerg and Mojn (DNK) stations surveying
- [Galileo](./GAL):
  - Esbjerg and Mojn (DNK) stations surveying
- [BeiDou](./BDS):
  - Esbjerg and Mojn (DNK) stations surveying
- [BeiDou (GEO)](./BDS-GEO):
  - Esbjerg and Mojn (DNK) stations surveying
  - Navigation with GEO augmentation
- [Galileo with SBAS augmentation](./GAL_SBAS)
- [JMF: sampled by J.M. Friedt @ femto-st.fr (lab agency)](./JMF)
  - 2024-092 Mobile phone observations (Paris/urban)
- [SP3](./SP3)
  - Examples with SP3 data only or SP3 specific applications
- [48H](./48H)
  - 48h surveying (2 day course with double RINEX)
- [RINEX(A) - RINEX(B): differential analysis](./DIFF)
  - Esbjerg and Mojn (DNK): close range observations
- [Meteo RINEX applications](./METEO)
  - Complete modern Meteo observations (QC)
- [Clock RINEX applications](./METEO)
  - Precise Clock products
  - PPP Navi with Clock or SP3 only (example)
- [Batch Time binning](./TBIN)
  - Esbjerg (24h/DNK) station observations
  - SP3 time binning
- [Batch splitting](./SPLIT)
  - Esbjerg (24h/DNK) station observations
  - SP3 batch splitting
- [QC](./QC) 
  - other QC and analysis examples (without surveying)
- [CSV](./CSV)
  - RINEX or SP3 to CSV extraction examples, typically
  used to forward to other software toolkits.
- [FILEGEN](./FILEGEN)
  - RINEX and other format synthesis, after running custom operations (input format preservation)
