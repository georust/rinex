V0.17 (-rc)
===========

The repo enters V0.17 validation stage.  

### Update

- The crate features are documented in `Cargo.toml` in the form of `# Comments`
- Broadcast radio navigation (ephemeris calculations) have been validated and tested
for BeiDou.
- Although SBAS navigation is not fully supported yet, a few steps were taken toward
complet support.

- The QC library has been vastly improved. The geodetic repports look & feel is improved,
the inner workings are improved. It facilitates future improvements. 
Also, Graphs are directly incoporated to the geodetic reports, which make them complete and our
unique output product. Custom extra chapters can be added to add PVT solutions for example.

- Refactor of the inner folders inside `rinex/`
  - all RINEX types follow the same architecture. 
  For example, `parsing.rs` integrates the parsing logic.
  - improved features dependent architecture (less code and shorter files, clearer architecture)

- The RINEX and other internal libraries have been vastly simplified.
Although that did not involve major difficulties, it demanded a lof ot time, because
all RINEX types were simplified. In short, they are now reduced to 1D, of the form
`Map<K, V>`, while previous forms could be up to 3 or 4D. This will vastly simplify
the understanding of the inner objects, and it also facilitates post processing.

- Good progress towards Parsing / Formatting dual capability.
- The CRINEX infrastructure has been simplified and improved at the same time.
- Benchmarks are now fully integrated to Github CI, that includes
  - CRINEX decompression
  - RINEX parsing
  - A few post processing tasks
- The BINEX library appears (work in progress)
  - RNX2BIN APi and Applications appear: serve your RINEX to binary as BINEX (I/O or file.bin)
  - BIN2RNX APi and Applications appear: collect BINEX stream (I/O or dumped file) as RINEX
- RTCM options appear (Work in progress)
  - RNX2RTCM serve RINEX content as RTCM messages (I/O or file.bin)
- Improved overall API documentation (`docrs`) 
- Vastly improved testing ecosystem: more generic methods: code less but test more

### Breaking changes

- all RINEX iterators have been renamed to `_iter()`, to follow standard naming conventions
- `rinex::prelude` delivers inner sub categories. For example, the ANISE and Ephemeris oriented
exports are delivered by `rinex::prelude::nav`.
