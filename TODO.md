Roadmap
=======

- [ ] Epoch:
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
- [ ] Processing: conclude MP bias analysis
- [ ] Processing: remove Wl and Nl combinations,
merge them into a unique MW combination, and we must form the exact
same combinations on both Wl/Nl sides
- [ ] compression & decompression benchmarking
- [ ] IONEX: merge operation
- [ ] Misc
  - [ ] provide python bindings, similarly to `Hifitime`.  
   Probably focus on high level and most common methods ?  
   Python bindings should be a crate "feature"
  - [ ] CI: automatic building of binaries for easy download by non-developers.  
   
  - features are not exposed to the API, we should at least
  exhibit which features exist and what they provide

  - Enhance reader/writer with hatanaka capacity to simplify file operations ?
  - Implement Lines<BufReader> iterator ourselves and avoid its memory allocation
  that takes place at every single line iteration
  
- [ ] Performances
  - [ ] convert string.find() to regex.find()
  - [ ] use Cow when possible

- [ ] CLI
  - [ ] time binning, TEQC op
  - [ ] improve vehicle color map (sv identification).
  PRN close to one another (like G08 and G09) produce a color too close
  to one another. It becomes hard to tell who is who.
  We simply need a broader color space
  - [ ] improve time axis rendering by converting to a range of date
  - [ ] When Observation + Navigation context is combined,
  it would be useful to add the Elevation angles against observation plots.
  To do this, we need to enable a secondary Y axis, scale to encountered elevation angles.
  - [ ] When Observations came with receiver clock offsets,
  it would be nice to evaluate its drift like we do in `--qc` mode, 
  and plot it.
  - [ ] emphasize Observation EpochFlags and external events,
  with like a plot annotations or something like that
  - [ ] progress towards quality check `--qc`
  - [ ] Find an efficient method to customize header fields
- [ ]  Data production
  - [ ] provide some interface to efficiently customize the Header section
  - [ ] provide an efficient interface to manage file names to be generated 
  - [ ] Make GUI an application feature? for users not interested in such option

- [ ] Ublox
  - [ ] Have a header field attributes customization interface similar to `cli` application
  - [ ] Generate Observation Data (requires `observation::to_file` to be completed)
  - [ ] Generate Ephemeris Data (requires `navigation::to_file` to be completed)
