Scripts
=======

List of Shell scripts to operate our applications (no GUI available yet) and perform interesting tasks.

## Getting started

All scripts are intended to be executed at the base of this Git repository,
and expect the `released` binaries to have been compiled

```bash
cd georust-rinex
cargo build --release --all-features

# Now try one of the survey scripts, for example:
./tutorials/GAL/mojdnk.sh
```

RINEX-Cli and other applications will generate logs but we do not store them in the following examples.

## Surveying

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
- [RINEX(A) - RINEX(B): differential analysis](./DIFF)
  - Esbjerg and Mojn (DNK): close range observations
- [Meteo observations exploitation](./METEO)
  - Complete modern Meteo observations (QC)
- [Time binning: time reframing examples](./TBIN)
  - Esbjerg (24h/DNK) station observations
  - SP3 time binning
