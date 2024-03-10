//! RINEX compression module
use super::{numdiff::NumDiff, textdiff::TextDiff, Error};
use crate::is_rinex_comment;
use crate::{Constellation, Observable, SV};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Default, PartialEq)]
pub enum State {
    #[default]
    EpochDescriptor,
    Body,
}

impl State {
    /// Resets Finite State Machine
    pub fn reset(&mut self) {
        *self = Self::default()
    }
}

/// Structure to compress RINEX data
pub struct Compressor {
    /// finite state machine
    state: State,
    /// True only for first epoch ever processed
    first_epoch: bool,
    /// epoch line ptr
    epoch_ptr: u8,
    /// epoch descriptor
    epoch_descriptor: String,
    /// flags descriptor being constructed
    flags_descriptor: String,
    /// vehicles counter in next body
    nb_vehicles: usize,
    /// vehicle pointer
    vehicle_ptr: usize,
    /// obs pointer
    obs_ptr: usize,
    /// Epoch differentiator
    epoch_diff: TextDiff,
    /// Clock offset differentiator
    clock_diff: NumDiff,
    /// Vehicle differentiators
    sv_diff: HashMap<SV, HashMap<usize, (NumDiff, TextDiff, TextDiff)>>,
    /// Pending kernel re-initialization
    forced_init: HashMap<SV, Vec<usize>>,
}

fn format_epoch_descriptor(content: &str) -> String {
    let mut result = String::new();
    result.push('&');
    for line in content.lines() {
        result.push_str(line.trim()) // removes all \tab
    }
    result.push('\n');
    result
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            first_epoch: true,
            epoch_ptr: 0,
            epoch_descriptor: String::new(),
            flags_descriptor: String::new(),
            state: State::default(),
            nb_vehicles: 0,
            vehicle_ptr: 0,
            obs_ptr: 0,
            epoch_diff: TextDiff::new(),
            clock_diff: NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER).unwrap(),
            sv_diff: HashMap::new(),
            forced_init: HashMap::new(),
        }
    }
}

impl Compressor {
    /// Identifies amount of vehicles to be provided in next iterations
    /// by analyzing epoch descriptor
    fn determine_nb_vehicles(&self, content: &str) -> Result<usize, Error> {
        if content.len() < 33 {
            Err(Error::MalformedEpochDescriptor)
        } else {
            let nb = &content[30..32];
            if let Ok(u) = nb.trim().parse::<u16>() {
                //println!("Identified {} vehicles", u); //DEBUG
                Ok(u.into())
            } else {
                Err(Error::MalformedEpochDescriptor)
            }
        }
    }

    /// Identifies vehicle from previously stored epoch descriptor
    fn current_vehicle(&self, constellation: &Constellation) -> Result<SV, Error> {
        let sv_size = 3;
        let epoch_size = 32;
        let vehicle_offset = self.vehicle_ptr * sv_size;
        let min = epoch_size + vehicle_offset;
        let max = min + sv_size;
        let vehicle = &mut self.epoch_descriptor[min..max].trim().to_string();
        if let Some(constell_id) = vehicle.chars().nth(0) {
            if constell_id.is_ascii_digit() {
                // in old RINEX + mono constell context
                //   it is possible that constellation ID is omitted..
                vehicle.insert_str(0, &format!("{:x}", constellation));
            }
            let sv = SV::from_str(vehicle)?;
            //println!("VEHICULE: {}", sv); //DEBUG
            Ok(sv)
        } else {
            Err(Error::VehicleIdentificationError)
        }
    }

    /// Concludes current vehicle
    fn conclude_vehicle(&mut self, content: &str) -> String {
        let mut result = content.to_string();
        //println!(">>> VEHICULE CONCLUDED"); //DEBUG
        // conclude line with lli/ssi flags
        let flags = self.flags_descriptor.trim_end();
        if !flags.is_empty() {
            result.push_str(flags);
        }
        result.push('\n');
        self.flags_descriptor.clear();
        // move to next vehicle
        self.obs_ptr = 0;
        self.vehicle_ptr += 1;
        if self.vehicle_ptr == self.nb_vehicles {
            self.conclude_epoch();
        }
        result
    }

    /// Concludes current epoch
    fn conclude_epoch(&mut self) {
        //DEBUG
        //println!(">>> EPOCH CONCLUDED \n");
        self.epoch_ptr = 0;
        self.vehicle_ptr = 0;
        self.epoch_descriptor.clear();
        self.state.reset();
    }

    /// Schedule given kernel for reinitizalition
    /// due to omitted data field.
    /// We only do so if kernel was previously initialized
    fn schedule_kernel_init(&mut self, sv: SV, index: usize) {
        if let Some(indexes) = self.sv_diff.get(&sv) {
            if indexes.contains_key(&index) {
                if let Some(indexes) = self.forced_init.get_mut(&sv) {
                    if !indexes.contains(&index) {
                        indexes.push(index);
                    }
                } else {
                    self.forced_init.insert(sv, vec![index]);
                }
                //DEBUG
                //println!("PENDING: {:?}", self.forced_init);
            }
        }
    }

    /// Compresses given RINEX data to CRINEX
    pub fn compress(
        &mut self,
        _rnx_major: u8,
        observables: &HashMap<Constellation, Vec<Observable>>,
        constellation: &Constellation,
        content: &str,
    ) -> Result<String, Error> {
        let mut result: String = String::new();
        let mut lines = content.lines();

        loop {
            let line: &str = match lines.next() {
                Some(l) => {
                    if l.trim().is_empty() {
                        // line completely empty
                        // ==> determine if we were expecting content
                        if self.state == State::Body {
                            // previously active
                            if self.obs_ptr > 0 {
                                // previously active
                                // identify current SV
                                if let Ok(sv) = self.current_vehicle(constellation) {
                                    // nb of obs for this constellation
                                    let sv_nb_obs = observables[&sv.constellation].len();
                                    let nb_missing = std::cmp::min(5, sv_nb_obs - self.obs_ptr);
                                    //println!("Early empty line - missing {} field(s)", nb_missing); //DEBUG
                                    for i in 0..nb_missing {
                                        result.push(' '); // empty whitespace, on each missing observable
                                                          // to remain retro compatible with official tools
                                        self.flags_descriptor.push_str("  "); // both missing
                                        self.schedule_kernel_init(sv, self.obs_ptr + i);
                                    }
                                    self.obs_ptr += nb_missing;
                                    if self.obs_ptr == sv_nb_obs {
                                        // vehicle completion
                                        result = self.conclude_vehicle(&result);
                                    }

                                    if nb_missing > 0 {
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    l
                },
                None => break, // done iterating
            };

            // println!("\nWorking from LINE : \"{}\"", line); //DEBUG

            // [0] : COMMENTS (special case)
            if is_rinex_comment(line) {
                if line.contains("RINEX FILE SPLICE") {
                    // [0*] SPLICE special comments
                    //      merged RINEX Files
                    self.state.reset();
                    //self.pointer = 0
                }
                result // feed content as is
                    .push_str(line);
                result // \n dropped by .lines()
                    .push('\n');
                continue;
            }

            match self.state {
                State::EpochDescriptor => {
                    if self.epoch_ptr == 0 {
                        // 1st line
                        // identify #systems
                        self.nb_vehicles = self.determine_nb_vehicles(line)?;
                    }
                    self.epoch_ptr += 1;
                    self.epoch_descriptor.push_str(line);

                    //TODO
                    //pour clock offsets
                    /*if line.len() > 60-12 {
                        Some(line.split_at(60-12).1.trim())
                    } else {
                        None*/
                    //TODO
                    // if we did have clock offset,
                    //  append in a new line
                    //  otherwise append a BLANK
                    self.epoch_descriptor.push('\n');

                    let nb_lines = num_integer::div_ceil(self.nb_vehicles, 12) as u8;
                    if self.epoch_ptr == nb_lines {
                        // end of descriptor
                        // format to CRINEX
                        self.epoch_descriptor = format_epoch_descriptor(&self.epoch_descriptor);
                        if self.first_epoch {
                            //println!("INIT EPOCH with \"{}\"", self.epoch_descriptor); //DEBUG
                            self.epoch_diff.init(&self.epoch_descriptor);
                            result.push_str(&self.epoch_descriptor);
                            /////////////////////////////////////
                            //TODO
                            //missing clock offset field here
                            //next line should not always be empty
                            /////////////////////////////////////
                            result.push('\n');
                            self.first_epoch = false;
                        } else {
                            result.push_str(
                                self.epoch_diff.compress(&self.epoch_descriptor).trim_end(),
                            );
                            result.push('\n');
                            /////////////////////////////////////
                            //TODO
                            //missing clock offset field here
                            //next line should not always be empty
                            /////////////////////////////////////
                            result.push('\n');
                        }

                        self.obs_ptr = 0;
                        self.vehicle_ptr = 0;
                        self.flags_descriptor.clear();
                        self.state = State::Body;
                    }
                },
                State::Body => {
                    // nb of obs in this line
                    let nb_obs_line = num_integer::div_ceil(line.len(), 17);
                    // identify current satellite using stored epoch description
                    if let Ok(sv) = self.current_vehicle(constellation) {
                        // nb of obs for this constellation
                        let sv_nb_obs = observables[&sv.constellation].len();
                        if self.obs_ptr + nb_obs_line > sv_nb_obs {
                            // facing an overflow
                            // this means all final fields were omitted,
                            // ==> handle this case
                            //println!("SV {} final fields were omitted", sv); //DEBUG
                            for index in self.obs_ptr..sv_nb_obs + 1 {
                                self.schedule_kernel_init(sv, index);
                                result.push(' '); // put an empty space on missing observables
                                                  // this is how RNX2CRX (official) behaves,
                                                  // if we don't do this we break retro compatibility
                                self.flags_descriptor.push_str("  ");
                            }
                            result = self.conclude_vehicle(&result);
                            if self.state == State::EpochDescriptor {
                                // epoch got also concluded
                                // --> rewind fsm
                                self.nb_vehicles = self.determine_nb_vehicles(line)?;
                                self.epoch_ptr = 1; // we already have a new descriptor
                                self.epoch_descriptor.push_str(line);
                                self.epoch_descriptor.push('\n');
                                continue; // avoid end of this loop,
                                          // as this vehicle is now concluded
                            }
                        }

                        // compress all observables
                        // and store flags for line completion
                        let mut observables = line;
                        for _ in 0..nb_obs_line {
                            let index = std::cmp::min(16, observables.len()); // avoid overflow
                                                                              // as some data flags might be omitted
                            let (data, rem) = observables.split_at(index);
                            let (obsdata, flags) = data.split_at(14);
                            observables = rem;
                            if let Ok(obsdata) = obsdata.trim().parse::<f64>() {
                                let obsdata = f64::round(obsdata * 1000.0) as i64;
                                if flags.trim().is_empty() {
                                    // Both Flags ommited
                                    //println!("OBS \"{}\" LLI \"X\" SSI \"X\"", obsdata); //DEBUG
                                    // data compression
                                    if let Some(sv_diffs) = self.sv_diff.get_mut(&sv) {
                                        // retrieve observable state
                                        if let Some(diffs) = sv_diffs.get_mut(&self.obs_ptr) {
                                            let compressed: i64;
                                            // forced re/init is pending
                                            if let Some(indexes) = self.forced_init.get_mut(&sv) {
                                                if indexes.contains(&self.obs_ptr) {
                                                    // forced reinit pending
                                                    compressed = obsdata;
                                                    diffs.0.init(3, obsdata).unwrap();
                                                    diffs.1.init(" ");
                                                    diffs.2.init(" ");
                                                    //println!("FORCED REINIT WITH FLAGS \"{}\"", self.flags_descriptor); //DEBUG
                                                    result.push_str(&format!("3&{} ", compressed)); //append obs
                                                                                                    // remove from pending list,
                                                                                                    // so we only force it once
                                                    for i in 0..indexes.len() {
                                                        if indexes[i] == self.obs_ptr {
                                                            indexes.remove(i);
                                                            break;
                                                        }
                                                    }
                                                    if indexes.is_empty() {
                                                        self.forced_init.remove(&sv);
                                                    }
                                                } else {
                                                    // compress data
                                                    compressed = diffs.0.compress(obsdata);
                                                    result.push_str(&format!("{} ", compressed));
                                                    //append obs
                                                }
                                            } else {
                                                // compress data
                                                compressed = diffs.0.compress(obsdata);
                                                result.push_str(&format!("{} ", compressed));
                                                //append obs
                                            }

                                            let _ = diffs.1.compress(" ");
                                            let _ = diffs.2.compress(" ");
                                            // ==> empty flags fields
                                            self.flags_descriptor.push_str("  ");
                                        } else {
                                            // first time dealing with this observable
                                            let mut diff: (NumDiff, TextDiff, TextDiff) = (
                                                NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)?,
                                                TextDiff::new(),
                                                TextDiff::new(),
                                            );
                                            diff.0.init(3, obsdata).unwrap();
                                            result.push_str(&format!("3&{} ", obsdata)); //append obs
                                            diff.1.init(" "); // BLANK
                                            diff.2.init(" "); // BLANK
                                            self.flags_descriptor.push_str("  ");
                                            sv_diffs.insert(self.obs_ptr, diff);
                                        }
                                    } else {
                                        // first time dealing with this vehicle
                                        let mut diff: (NumDiff, TextDiff, TextDiff) = (
                                            NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)?,
                                            TextDiff::new(),
                                            TextDiff::new(),
                                        );
                                        diff.0.init(3, obsdata).unwrap();
                                        result.push_str(&format!("3&{} ", obsdata)); //append obs
                                        diff.1.init(" "); // BLANK
                                        diff.2.init(" "); // BLANK
                                        self.flags_descriptor.push_str("  ");
                                        let mut map: HashMap<usize, (NumDiff, TextDiff, TextDiff)> =
                                            HashMap::new();
                                        map.insert(self.obs_ptr, diff);
                                        self.sv_diff.insert(sv, map);
                                    }
                                } else {
                                    //flags.len() >=1 : Not all Flags ommited
                                    let (lli, ssi) = flags.split_at(1);
                                    //println!("OBS \"{}\" - LLI \"{}\" - SSI \"{}\"", obsdata, lli, ssi); //DEBUG
                                    if let Some(sv_diffs) = self.sv_diff.get_mut(&sv) {
                                        // retrieve observable state
                                        if let Some(diffs) = sv_diffs.get_mut(&self.obs_ptr) {
                                            // compress data
                                            let compressed: i64;
                                            // forced re/init is pending
                                            if let Some(indexes) = self.forced_init.get_mut(&sv) {
                                                if indexes.contains(&self.obs_ptr) {
                                                    // forced init pending
                                                    compressed = obsdata;
                                                    result.push_str(&format!("3&{} ", compressed));
                                                    diffs.0.init(3, obsdata).unwrap();
                                                    // remove from pending list,
                                                    // so we only force it once
                                                    for i in 0..indexes.len() {
                                                        if indexes[i] == self.obs_ptr {
                                                            indexes.remove(i);
                                                            break;
                                                        }
                                                    }
                                                    if indexes.is_empty() {
                                                        self.forced_init.remove(&sv);
                                                    }
                                                } else {
                                                    compressed = diffs.0.compress(obsdata);
                                                    result.push_str(&format!("{} ", compressed));
                                                }
                                            } else {
                                                compressed = diffs.0.compress(obsdata);
                                                result.push_str(&format!("{} ", compressed));
                                            }

                                            let lli = diffs.1.compress(lli);
                                            self.flags_descriptor.push_str(&lli);

                                            let ssi = diffs.2.compress(ssi);
                                            self.flags_descriptor.push_str(&ssi);
                                        } else {
                                            // first time dealing with this observable
                                            let mut diff: (NumDiff, TextDiff, TextDiff) = (
                                                NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)?,
                                                TextDiff::new(),
                                                TextDiff::new(),
                                            );
                                            diff.0.init(3, obsdata).unwrap();
                                            diff.1.init(lli);
                                            diff.2.init(ssi);
                                            result.push_str(&format!("3&{} ", obsdata)); //append obs
                                            if !lli.is_empty() {
                                                self.flags_descriptor.push_str(lli);
                                            } else {
                                                self.flags_descriptor.push(' ');
                                            }

                                            if !ssi.is_empty() {
                                                self.flags_descriptor.push_str(ssi);
                                            } else {
                                                // SSI omitted
                                                self.flags_descriptor.push(' ');
                                            }
                                            sv_diffs.insert(self.obs_ptr, diff);
                                        }
                                    } else {
                                        // first time dealing with this vehicle
                                        let mut diff: (NumDiff, TextDiff, TextDiff) = (
                                            NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)?,
                                            TextDiff::new(),
                                            TextDiff::new(),
                                        );
                                        diff.0.init(3, obsdata).unwrap();
                                        result.push_str(&format!("3&{} ", obsdata)); //append obs
                                        diff.1.init(lli);
                                        diff.2.init(ssi);
                                        self.flags_descriptor.push_str(lli);
                                        if !ssi.is_empty() {
                                            self.flags_descriptor.push_str(ssi);
                                        } else {
                                            // SSI omitted
                                            diff.2.init(" "); // BLANK
                                            self.flags_descriptor.push(' ');
                                        }
                                        let mut map: HashMap<usize, (NumDiff, TextDiff, TextDiff)> =
                                            HashMap::new();
                                        map.insert(self.obs_ptr, diff);
                                        self.sv_diff.insert(sv, map);
                                    }
                                }
                            } else {
                                //obsdata::f64::from_str()
                                // when floating point parsing is in failure,
                                // we know this observable is omitted
                                result.push(' '); // put an empty space on missing observables
                                                  // this is how RNX2CRX (official) behaves,
                                                  // if we don't do this we break retro compatibility
                                self.flags_descriptor.push_str("  ");
                                self.schedule_kernel_init(sv, self.obs_ptr);
                            }
                            self.obs_ptr += 1;
                            //println!("OBS {}/{}", self.obs_ptr, sv_nb_obs); //DEBUG

                            if self.obs_ptr > sv_nb_obs {
                                // unexpected overflow
                                return Err(Error::MalformedEpochBody); // too many observables were found
                            }
                        } //for i..nb_obs in this line

                        if self.obs_ptr == sv_nb_obs {
                            // vehicle completion
                            result = self.conclude_vehicle(&result);
                        }
                    } else {
                        // sv::from_str()
                        // failed to identify which vehicle we're dealing with
                        return Err(Error::VehicleIdentificationError);
                    }
                },
            } //match(state)
        } //main loop
        result.push('\n');
        Ok(result)
    }
    //notes:
    //si le flag est absent: "&" pour insérer un espace
    //tous les flags sont foutus a la fin en guise de dernier mot
}
