Roadmap
=======

- [ ] Epoch:
  - [x] convert `chrono::Duration` to `hifitime::Epoch` to describe the sampline timestamp
  - [ ] when parsing a record, `flag::HeaderInformationFollows` is not exploited,
the following content is probably interpreted as a faulty epoch to disregard

- [ ] Data production
  - [ ] Navigation 
  - [ ] Clock
  - [ ] IONEX
  - [ ] ANTEX 

- [ ] Observation
  - [ ] support possible data scaling
  for high precision RINEX
  - [ ] support scaling properly even in case of CRINEX
  - [ ] possible receiver clock offset compensation
  table 5 p7 RINEX4

- [ ] Data decompression
  - [ ] improve compression & decompression testbenches
- [ ] introduce compression & decompression benchmarking

- [ ] Misc
  - provide python bindings, similarly to `Hifitime`.  
   Probably focus on high level and most common methods ?  
   Python bindings should be a crate "feature"
   
  - features are not exposed to the API, we should at least
  exhibit which features exist and what they provide

  - Enhance reader/writer with hatanaka capacity to simplify file operations ?
  - Implement Lines<BufReader> iterator ourselves and avoid its memory allocation
  that takes place at every single line iteration
  
- [ ] Performances
  - [ ] convert string.find() to regex.find()
  - [ ] use Cow when possible

- [ ] CLI
  - [ ] improve vehicle color map (sv identification)
  - [ ] improve time axis by converting to a proper date axis
  - [ ] emphasize Observation EpochFlags and external events,
  with like a plot annotation or something like that
  - [ ] progress towards quality check 
  - [ ] conclude the `teqc` mini ascii plot 
  - [ ] `teqc` like verbose / analysis report ? 
  - [ ] Find an efficient method to customize header fields
- [ ]  Data production
  - [ ] provide some interface to efficiently customize the Header section
  - [ ] provide an efficient interface to manage file names to be generated 
  - [ ] Make GUI an application feature? for users not interested in such option

- [ ] Ublox
  - [ ] Have an header field attributes customization interface similar to `cli` application
  - [ ] Generate Observation Data (requires `observation::to_file` to be completed)
  - [ ] Generate Ephemeris Data (requires `navigation::to_file` to be completed)

Completed
=========

- [x] Epoch:
  - [x] convert `chrono::Duration` to `hifitime::Epoch` to describe the sampline timestamp

- File operations
  - [x] Provide a Writer wrapper in similar fashion to existing Reader wrapper for efficient data compression
- RINEX Record
  - [x] browsing documentation integrated to the API
- Data decompression
  - [x] CRX2RNX CRX1 thorough test
  - [x] Conclude [numerical data compression](https://github.com/gwbres/rinex/blob/main/rinex/src/hatanaka.rs#L164)
- Data Production
  - [x] Observation V2, V3, V4
  - [x] Meteo, V2,V3,V4
  - [x] Find an efficient test method. Test method parses a given file from the test pool,
  then produces a copy, and evaluates rnx.eq(copy) which must not fail.
  This is powerful because it is a bitwise comparison and takes truncation into account.
  This naturally takes care of possible Header section and vehicules reordering.
  The only things that are not tested, are header fields we are unable to parse, but that is not important.
  If test failed, we print the result of diff(initial, copy) to debug, to be found in the CI logs.
- Data compression 
  - [x] Conclude [text data compression](https://github.com/gwbres/rinex/blob/main/rinex/src/hatanaka.rs#L209)
  - [x] Verify data scaling is correctly restablish in decompression
  - [x] Adjust production method to take advantage of newly available Writer wrapper
  - [x] Unlock `CRINEX` data production
  - [x] Data conversion and scaling 
  Compression & Decompression kernels cast and interprate data themselves correctly.
  Everything is coded on signed 64b data, we do not face the same types of issues that official
  compression / decompression tools may face
  - [x] `Gzip`  "invalid gzip header" decompression failure
The previous file pointer operation was not correct.
This was due to some inquiries trying to make the decompressor knowledgeable of
Hatanaka decompression. Currently this is not the case, we manage this scenario outside 
of the file browser

- `navigation` - `dictionary`
  - [x] General orbits health 
  - [x] GLO/Orbit2 channel #
  - [x] GLO/NAV4/Orbit7 status flag
- Post Processing
  - [x] 1D post processing [1D diff()](https://github.com/gwbres/rinex/blob/main/rinex/src/lib.rs#L3023) 

- CLI 
  - [x] provide efficient interface to 1D and 2D processing methods  
  - [x] Provide a visualization method when we're not generating a file
  - [x] Conclude Observation RINEX plotting

- Performances
  - [x] `hatanaka::TextDiff`
