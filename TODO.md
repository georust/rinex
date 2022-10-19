Roadmap
=======

## RINEX library

- [ ] `epoch` : `EpochFlag::HeaderInformationFollows` is not exploited to this day.  
We might want to update the Header structure, on the fly, with following information
- [ ] `sampling`: `chrono::duration` is used most of the time to describe a duration.  
The fractional parts ("nanos") is totally unused, we cannot handle periods smaller than 1 second to this day
- [ ] record browsing documentation for each Record type declaration
- [ ] Data production
  - [ ]  Major data production
    - [ ] Observation data production
    - [ ] Navigation data production
  - [ ] Minor data production
    - [ ] Clock data production 
    - [ ] Ionosphere maps production   
    - [ ] Antenna data production 
- [ ] Data decompression
  - [ ] CRX32RNX thorough test
- [ ] Post Processing
  - [ ] Conclude the 2D Post processing "double diff"
  - [ ] Add thorough test bench
    - [ ] A NAV + OBS context structure could help ?   
    this is currently inquired in the `differential` branch
  - [ ] Calculations involved in RTK solver? I am not familiar with such calculations
- Misc
  - Enhance reader/writer with hatanaka capacity to simplify file operations ?
  - Implement Lines<BufReader> iterator ourselves and avoid its memory allocation
  that takes place at every single line iteration
- Performances
  - [x] introduce benchmarking
  - [ ] improve TextDiff
  replace .find() by regex.find
  Cow ?
  - [ ] improve decompression method
  - [ ] improve parser
  - [ ] improve compression method

## Command Line application

- [ ] CLI
  - [ ] expose remaining interesting methods ?
  - [ ] conclude the `teqc` mini ascii plot 
  - [ ] Find an efficient method to customize header fields
  - [ ] We are limited to "clap" 3.x as long as we use a "yaml"
  command line description
- [ ]  Data production
  - [ ] provide some interface to efficiently customize the Header section
  - [ ] provide an efficient interface to manage file names to be generated 
- [ ]  Post Processing
  - [ ]  provide efficient interface to 1D and 2D processing methods  
- [ ] Graphical Interface
  - [ ] Conclude Observation RINEX plotting
  - [ ] Provide NAV and MET RINEX plotting 
  - [ ] Make GUI an application feature? for users not interested in such option

## UBLOX application

- [ ] Have an header field attributes customization interface similar to `cli` application
- [ ] Generate Observation Data (requires `observation::to_file` to be completed)
- [ ] Generate Ephemeris Data (requires `navigation::to_file` to be completed)

## Completed

- File operations
  - [x] Provide a Writer wrapper in similar fashion to existing Reader wrapper for efficient data compression
- Data decompression
  - [x] CRX2RNX CRX1 thorough test
  - [x] Conclude [numerical data compression](https://github.com/gwbres/rinex/blob/main/rinex/src/hatanaka.rs#L164)
- Data Production
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

- Graphical Interface
  - [x] Provide a visualization method when we're not generating a file
