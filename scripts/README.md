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
./scripts/GAL/mojdnk-cpp.sh
```

Surveying and other RINEX-Cli applications will generate logs that we store in a temporary logs/ folder,
so you can further inquire what happened during the process.

## Surveying

Field surveying (currently only static) aims at determining the position of
reference stations very precisely without a priori knowledge, so they can serve later on as reference stations for differential
positioning techniques.

Most scripts are sorted by constellation being used in the navigation/surveying or analysis process, either
single or a combination of constellations might be used, possibly SBAS service too.

When we say BRDC, it emphasizes _real time_ surveying based solely on radio messages, 
as opposed to _post-processed_ surveying which uses higher accuracy products and exhibits better results.

CGGTTS solutions are special timing oriented solutions, to compare remote clocks to one another. 
We demonstrate the synthesis of CGGTTS solutions along our position surveys.

Other file operations are also demonstrated in this repo.

- [GPS](./GPS):
  - Esbjerg and Mojn (DNK) stations surveying
- [Galileo](./GAL):
  - Esbjerg and Mojn (DNK) stations surveying
- [BeiDou](./BDS):
  - Esbjerg and Mojn (DNK) stations surveying
- [Galileo with SBAS augmentation](./GAL_SBAS)
- [JMF: sampled by J.M. Friedt @ femto-st.fr (lab agency)](./JMF)
  - 2024-092 Mobile phone observations (Paris/urban)
- [RINEX(A) - RINEX(B) Differential analysis illustration](./DIFF)
  - Esbjerg and Mojn (DNK): close range observations
- [Time binning / time reframing examples](./TBIN)
  - Esbjerg (24h/DNK) station observations
  - SP3 time binning
