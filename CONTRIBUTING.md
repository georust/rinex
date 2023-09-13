CONTRIBUTING
============

Hello and welcome on board :wave:

Use the github portal to submit PR, all contributions are welcomed.  
Don't forget to run a quick `cargo fmt` prior any submissions, so the CI/CD does not fail
on coding style "issues".

This crate and ecosystem is part of the Georust community. 
You can contact us on [Discord](https://discord.gg/Fp2aape).

Crate architecture
==================

For each supported RINEX types, we have one folder named after that format.  
`src/navigation` is one of those. 

The module contains the record type definition. RINEX file contents vary a lot
depending on which type of RINEX we're talking about.  
For complex RINEX formats like Navigation Data, that module will contain all possible inner types.

Other important structures :
- `src/epoch/mod.rs`: the Epoch module basically provides
hifitime::Epoch parsing methods, because RINEX describes date in non standard formats.
Also, the `Flag` structure is used to mark Observations (valid or invalid).
- `src/constellation.rs` defines GNSS constellations
- `src/constellation/augmentation.rs` : preliminary SBAS support
- `src/sv.rs` defines a Satellite vehicle, which is associated to a constellation
- `src/observable.rs`: defines possible observations like raw phase
- `src/carrier.rs`: defines carrier signals in terms of frequency and bandwidth.
It also contains utilities to identify which GNSS signals we're dealing with,
from an `Observable`.
- `src/hatanaka/mod.rs`: the Hatanaka module contains the RINEX Compressor and Decompressor 
- `src/antex/antenna.rs`: defines the index structure of ANTEX format

NAV RINEX
=========

Orbit instantaneous parameters, broadcasted by GNSS vehicles, are presented in different
forms depending on the RINEX revision and the GNSS constellation.

To solve that problem, we use a dictionary, in the form of `src/db/NAV/orbits.json`,
which describes all fields per RINEX revision and GNSS constellation.

This file is processed at build time, by the build script and ends up as a static 
pool we rely on when parsing a file. 

The dictionary is powerful enough to describe all revision, the default Navigation Message
is `LNAV`: Legacy NAV message, which can be omitted. Therefore, it is only declared 
for modern Navigation messages.

Introducing a new RINEX type
============================

`src/meteo/mod.rs` is the easiest format and can serve as a guideline to follow.

When introducing a new Navigation Data, the dictionary will most likely have to be updated (see previous paragraph).
