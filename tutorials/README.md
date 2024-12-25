Tutorials
=========

The tutorials serie will try to illustrate all options offered by the toolbox.

Prior running our examples, you are expected 
[to read how the two Wiki pages](https://github.com/georust/wiki)

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
- [IONex](./IONex) format specific examples
  - Resampling
  - TEC map projection
- [JMF: sampled by J.M. Friedt @ femto-st.fr](./JMF) french lab
  - 2024-092 Mobile phone observations (Paris/urban)
  - 2024-110 Nyalesund (NOR) 2024 glacier surveying (RTK compatible)
  - 2024-111 Nyalesund (NOR) 2024 glacier surveying (RTK compatible)
- [Merge](./MERGE) special file operation
  - Merge (RINex (a), RINex (b)) into a new RINex
  - Merge (IONex (a), IONex (b)) into a new IONex
  - Merge (SP3 (a), SP3 (b)) into a new SP3
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
