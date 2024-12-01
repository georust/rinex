V0.17 (-rc)
===========

The repo enters V0.17 validation stage.  
A huge phase of library simplification was undertaken, it took quite some time because
this repo contains a lot of stuff, but that did not involve difficulties.

Starging from V0.17 (and its `-rc`), the RINEX, SP3 and QC API (libraries)
have been vastly simplified, to help new contributions and facilitate post processing.

### Update

- The crate features are documented in `Cargo.toml` in the form of `# Comments`
- Broadcast radio navigation (ephemeris calculations) have been validated and tested
- Major steps to SBAS augmented navigation, yet not fully completed.

- The QC library has been vastly improved. Currently, it renders a HTML report
into the workspace (this is our toolkit behavior), which may be improved (if that proves useful) in near future.
  - The HTML reports now integrate the Plotly graphs. The geodetic report is our unique User Interface
  - Huge progress towards real and meaningful geodetic reports.
  Sorted by physics, navigation report are presented per Constellation, possibility
  to add SP3 for PPP, possibility to consider the PVT (post processed navigation) and CGGTTS
  (special solutions) in the same report, to make it single and complete.

- Refactor of the inner folders inside `rinex/`
  - all RINEX types follow the same architecture. 
  For example, `parsing.rs` integrates the parsing logic.
  - improved features dependent architecture (less code and shorter files, clearer architecture)
- Good progress towards Parsing / Dumping dual capability.
- The CRINEX infrastructure has been simplified and improved at the same time.
- Benchmarks are now fully integrated to Github CI, that includes
  - CRINEX decompression
  - RINEX parsing
  - A few post processing tasks
- The BINEX library appears. Yet, it is not integrated to the toolkit.
It is not 100% clear yet what would be the best approach to introduce "real-time" oriented interfaces,
to our post-processing oriented applications.
- Improved overall API documentation (`docrs`) 

### Breaking changes

- all RINEX iterators have been renamed to `_iter()`, to follow standard naming conventions
- `rinex::prelude` delivers inner sub categories. For example, the ANISE and Ephemeris oriented
exports are delivered by `rinex::prelude::nav`.
