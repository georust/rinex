Roadmap 
=======

## RINEX library

- [ ] `epoch` : `EpochFlag::HeaderInformationFollows` is not exploited to this day.  
We might want to update the Header structure, on the fly, with following information

- [ ] `navigation` - `dictionnary`: currently only supports floating point and string data to be identified and parsed.
  - [ ] we should allow other native types like boolean or integer numbers, as they are also specified in RINEX standards (at least of the latter)
  - [ ] an interface for `bitflags!` mapping would be ideal for Binary fields
  - [ ] an interface for complex and custom enums mapping would be also ideal and help data analysis 

- [ ] `sampling`: `chrono::duration` is used most of the time to describe a duration.  
The fractionnal parts ("nanos") is totally unused, we cannot handle periods smaller than 1 second to this day

- [ ] Data production
  - [ ]  Major data production
    - [ ] Observation data production
    - [ ] Navigation data production
  - [ ] Minor data production
    - [ ] Clock data production 
    - [ ] Ionosphere maps production   
    - [ ] Antenna data production 

- [ ] Data compression
  - [ ] Conclude [numerical data compression](https://github.com/gwbres/rinex/blob/main/rinex/src/hatanaka.rs#L164)
  - [ ] Conclude [text data compression](https://github.com/gwbres/rinex/blob/main/rinex/src/hatanaka.rs#L209)
  - [ ] Provide a Writer wrapper in similar fashion to existing Reader wrapper for efficient data compression
  - [ ] Adjust production method to take advantage of newly available Writter wrapper
  - [ ] Unlock `CRINEX` data production
  - [ ] `Gzip` decompression failure: understand current issue regarding files marked for `Post Processing`, 
track [opened issue](https://github.com/rust-lang/flate2-rs/issues/316)

- [ ] Post Processing
  - [ ] Conclude the 2D Post processing "double diff"
    - [ ] A NAV + OBS context structure could help ?   
    this is currently inquired in the `differential` branch
  - [ ] Calculations involved in RTK solver ? I am not familiar with such calculations

## Command Line application

- [ ] CLI
  - [ ] expose remaining interesting methods ?
  - [ ] conclude the `teqc` mini ascii plot 
- [ ]  Data production
  - [ ] provide some interface to efficiently customize the Header section
  - [ ] provide an efficient interface to manage file names to be generated 
- [ ]  Post Processing
  - [ ]  provide efficient interface to 1D and 2D processing methods  
- [ ] Graphical Interface
  - [ ] Provide a visualization method when we're not generating a file
  - [ ] Inquire which framework would be ideal: not too complex, full of features
  - [ ] GUI must be an application feature, for users not interested in such option

## UBLOX application

- [ ] Generate Observation Data (requires `observation::to_file` to be completed)
- [ ] Generate Ephemeris Data (requires `navigation::to_file` to be completed)

## Done

- [x] Rinex Post Processing
  - [x] 1D post processing [1D diff()](https://github.com/gwbres/rinex/blob/main/rinex/src/lib.rs#L3023) 
- [ ] UBLOX
  - [ ] I just had the `End of Epoch` UBX frame merged
