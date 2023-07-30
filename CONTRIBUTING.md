CONTRIBUTING
============

First of all: Hello and welcome on board!

Use `cargo fmt` prior submitting a pull request so the CI/CD does not fail on coding standards issues.

This crate and ecosystem is part of the Georust community.  
You can contact us to ask questions on our Discord channel, 
see the [community portal](https://github.com/georust/geo)

lib architecture
================

For each supported RINEX types, we have one folder.  
The folder bears the name of such RINEX type. In that folder,
you can find a `record.rs` file where that particular files content is described. 
It also contains its dedicated parsing methods.

- src/lib.rs : main library, `Rinex` definitions
- src/constellation/mod.rs : GNSS constellations definition
  - src/constellation/augmentation.rs : SBAS related definitions 
- src/sv.rs : Satellite vehicule definitions
- src/observation/mod.rs : OBS RINEX entry point
  - src/observation/record.rs : OBS RINEX specific record definitions
- src/navigation/mod.rs : NAV RINEX entry point
  - src/navigation/record.rs : NAV RINEX specific generic record definitions
  - src/navigation/ephemeris.rs : Ephemeris frames definition, parsing method and related calculations, like Kepler solving
  - src/navigation/ionmessage.rs : new ION frame definition and parsing methods
  - src/navigation/eopmessage.rs : new EOP frame definition and parsing methods
- src/meteo/mod.rs : Meteo RINEX entry point
  - src/meteo/record.rs : specific record definitions, including parsing methods
  - src/hatanaka/mod.rs : Compression / Decompression module 

NAV RINEX
=========

Abstraction for NAV files parsing is provided by the _navigation.json_ descriptor.  
This is how we describe all supported revisions and their data fields.  

Improving or updating NAV file parsing either means updating this database, 
or improving the way we rely on it. This is handle

Introducing a new RINEX type
============================

A good guideline would be to follow the src/meteo/mod.rs module, which is simple enough,
yet follows the basic principles previously explained.  

A more complex RINEX declination would be src/navigation, because the inner record entry
can have 3 or 4 different variations.
