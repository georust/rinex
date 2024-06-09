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

We currently sort them by constellation being used in the navigation/surveying process, either
single constellations, a combination of constellations and possibly SBAS augmentation.  

When we say BRDC, it emphasizes _real time_ surveying based solely on radio messages, 
as opposed to _post-processed_ surveying which uses higher accuracy products and exhibits better results.

- [GPS](./GPS):
  - Esbjerg and Mojn (DNK) stations
- [Galileo](./GAL):
  - Esbjerg and Mojn (DNK) stations
- [BeiDou](./BDS):
  - Esbjerg and Mojn (DNK) stations
- [Galileo + SBAS applications](./GAL_SBAS)
- [JMF: sampled by J. M. Friedt @ femto-st.fr (lab agency)](./JMF)
  - Mobile phone observations (Paris metropolitan)
