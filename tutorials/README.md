Scripts
=======

This serie helps and illustrates most of the capabilities of the software contained
in this repo and the ecosystem. We split the examples by topic. For each topic, you will find
at least one example.

Before trying to understand our examples, you should read the part of our Wiki
that explains [how to load your data into the toolbox](https://github.com/georust/wiki)

Getting started
===============

Our examples expect the binaries to have been generated with all features activated (heaviest form):

```bash
# download the toolbox and dataset
git clone https://github.com/georust/rinex
cd rinex

# build the tools
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

Tutorials
=========

Most scripts are indexed by constellation, sometimes by signal or modulation.  
When we say `-brdc`, we want to emphasize that navigation is performed using
decoded radio message, rather than using post processed laboratory products.  

This is also sometimes referred to as "real time" surveying, because it is exactly like
navigating in real-time, except that the radio messages were stored as RINEX files, which
allows to replay them later.

`-sum` scripts emphasize that the analysis is generated with the `-summary` option.

CGGTTS solutions are special timing oriented navigation solutions. Some of our surveys
are 100% dedicated to CGGTTS, sometimes we solve standard and CGGTTS solutions at the same time.

- [48H](./48H) two RINEX datasets at once and more
  - 48h surveying (2 day course with two RINEX at once)
- [BeiDou](./BDS) constellation post-processing
  - Esbjerg and Mojn (DNK) stations surveying
- [BeiDou (GEO)](./BDS-GEO) stationnary vehicles applications
  - Esbjerg and Mojn (DNK) stations surveying
  - Navigation with GEO augmentation
- [BINEX](./BINEX) demonstrations
  - The only open source real time oriented GNSS format
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
  - SP3 and Clock comparison (for PPP)
- [Config](./config) contains special presets that some tutorials may use.
For example, some tutorials have lower quality than others: we use
a specific preset stored here.
- [CSV](./CSV) 
  - RINEX to CSV extraction capabilites
  - SP3 to CSV extraction capabilities
- [CRX2RNX](./CRX2RNX) CRINEX decompression to readable RINEX (file I/O)
- [File Synthesis](./FILEGEN)
  - RINEX pre-processing and synthesis (File I/O)
  - SP3 pre-processing and format (File I/O)
- [Galileo](./GAL) constellation post-processing
  - Esbjerg and Mojn (DNK) stations surveying
- [Galileo with SBAS augmentation](./GAL_SBAS)
- [GPS](./GPS) constellation post-processing
  - Esbjerg and Mojn (DNK) stations surveying
- [Ionosphere](./IONO) examples
  - TEC from Observation RINEX projection
  - Observed TEC versus laboratory (IONEX) comparison
  - IPP projections
  - Reconstructed signal delay from observed signals
- [IONEX](./IONEX) format specific examples
  - Map projection
- [JMF: sampled by J.M. Friedt @ femto-st.fr](./JMF) french lab
  - 2024-092 Mobile phone observations (Paris/urban)
  - 2024-110 Nyalesund (NOR) 2024 glacier surveying (RTK compatible)
  - 2024-111 Nyalesund (NOR) 2024 glacier surveying (RTK compatible)
- [Meteo](./METEO) special RINEX applications
  - Modern Meteo observations (QC)
  - Meteo for accurate Troposphere Modelling in PPP
- [QC](./QC) (other)
  - other QC and analysis examples (without surveying)
- [OBS](./OBS) Simple Observation RINEX exploitation
- [SP3](./SP3) high precision orbit applications 
  - Examples with SP3 data only 
  - SP3 for PPP
- [RINEX(A) - RINEX(B)](./DIFF) simple yet efficient differential analysis
  - Esbjerg and Mojn (DNK): close range observations
- [RNX2CRX](./RNX2CRX) RINEX compression to compact CRINEX (file I/O)
- [Time binning](./TBIN) into a batch of files (File I/O)
  - Esbjerg (24h/DNK) station observations
  - SP3 batch generation
- [Time splitting](./SPLIT)
  - Esbjerg (24h/DNK) station observations
  - SP3 batch splitting
