Scripts
=======

List of Shell scripts to operate our applications (no GUI available yet) and perform interesting tasks.

## Getting started

All scripts are intended to be executed at the base of this Git repository,
and expect the `released` binaries to have been compiled

```bash
cd georust-rinex
cargo build --release --all-features
```

## Surveying

Field surveying (currently only static) aims at determining the position of
reference stations very precisely without a priori knowledge, so they can serve later on as reference stations for differential
positioning techniques.

We currently sort them by constellation being used in the navigation/surveying process, either
single constellations, a combination of constellations and possibly SBAS augmentation.

- [Galileo](./GAL):
  - Esbjerg and Mojn (DNK) stations
- [GPS](./GPS):
  - Esbjerg and Mojn (DNK) stations
- [BeiDou](./BDS):
  - Esbjerg and Mojn (DNK) stations
- [Galileo + SBAS](./GAL_SBAS)
