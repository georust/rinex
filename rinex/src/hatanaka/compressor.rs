//! RINEX compression module
use crate::sv::Sv;
use crate::header;
use crate::is_comment;
use crate::types::Type;
use std::str::FromStr;
use std::collections::HashMap;

use super::{
    Error,
    numdiff::NumDiff,
    textdiff::TextDiff,
};

#[derive(PartialEq)]
pub enum State {
    EpochDescriptor,
    Body,
}

impl Default for State {
    fn default() -> Self {
        Self::EpochDescriptor
    }
}

impl State {
    /// Resets Finite State Machine
    pub fn reset (&mut self) {
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
    /// vehicules counter in next body
    nb_vehicules: usize,
    /// vehicule pointer
    vehicule_ptr: usize,
    /// obs pointer
    obs_ptr: usize,
    /// Epoch differentiator 
    epoch_diff: TextDiff,
    /// Clock offset differentiator
    clock_diff: NumDiff,
    /// Vehicule differentiators
    sv_diff: HashMap<Sv, Vec<(NumDiff, TextDiff, TextDiff)>>,
    /// Pending kernel re-initialization
    forced_init: HashMap<Sv, Vec<usize>>,
}

fn format_epoch_descriptor (content: &str) -> String {
    let mut result = String::new();
    result.push_str("&");
    for line in content.lines() {
        result.push_str(line.trim()) // removes all \tab
    }
    result.push_str("\n");
    result
}
    
impl Compressor {
    /// Creates a new compression structure 
    pub fn new() -> Self {
        Self {
            first_epoch: true,
            epoch_ptr: 0,
            epoch_descriptor: String::new(),
            flags_descriptor: String::new(),
            state: State::default(),
            nb_vehicules: 0,
            vehicule_ptr: 0,
            obs_ptr: 0,
            epoch_diff: TextDiff::new(),
            clock_diff: NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)
                .unwrap(),
            sv_diff: HashMap::new(),
            forced_init: HashMap::new(),
        }
    }

    fn vehicule_completion (&mut self, content: &str) -> String {
        let mut result = content.to_string();
        //DEBUG
        println!("");
        // conclude line with lli/ssi flags
        result.push_str(&format!("{}\n", self.flags_descriptor));
        // move to next vehicule
        self.obs_ptr = 0;
        self.vehicule_ptr += 1;
        self.flags_descriptor.clear();
        if self.vehicule_ptr == self.nb_vehicules {
            self.conclude_epoch()
        }
        result
    }

    fn conclude_epoch (&mut self) {
        //DEBUG
        println!("\n");
        self.epoch_ptr = 0;
        self.vehicule_ptr = 0;
        self.epoch_descriptor.clear();
        self.state.reset();
    }
    
    /// Compresses given RINEX data to CRINEX 
    pub fn compress (&mut self, header: &header::Header, content: &str) -> Result<String, Error> {
        // Context sanity checks
        if header.rinex_type != Type::ObservationData {
            return Err(Error::NotObsRinexData) ;
        }
        
        // grab useful information for later
        let _rnx_version = &header.version;
        let obs = header.obs
            .as_ref()
            .unwrap();
        let obs_codes = &obs.codes; 
        /*let crinex = obs.crinex
            .as_ref()
            .unwrap();
        let crx_version = crinex.version;*/
        
        let mut result : String = String::new();
        let mut lines = content.lines();

        loop {
            let line: &str = match lines.next() {
                Some(l) => {
                    //DEBUG
                    if l.trim().len() == 0 {
                        // line completely empty
                        // ==> determine if we're facing an early empty line
                        if self.state == State::Body { // previously active
                            if self.obs_ptr > 0 { // previously active
                                // identify current Sv
                                let sv_size = 3;
                                let epoch_size = 32;
                                let vehicule_offset = self.vehicule_ptr * sv_size;
                                let min = epoch_size + vehicule_offset;
                                let max = min + sv_size;
                                let vehicule = &self.epoch_descriptor[min..max];
                                if let Ok(sv) = Sv::from_str(vehicule.trim()) {
                                    // nb of obs for this constellation
                                    let sv_nb_obs = obs_codes[&sv.constellation].len();
                                    let nb_missing = std::cmp::min(5, sv_nb_obs - self.obs_ptr);
                                    //DEBUG
                                    println!("Early empty line - missing {} field(s)", nb_missing);
                                    for i in 0..nb_missing { 
                                        self.flags_descriptor.push_str("  "); // both missing
                                        //schedule re/init
                                        if let Some(indexes) = self.forced_init.get_mut(&sv) {
                                            indexes.push(self.obs_ptr+1+i);
                                        } else {
                                            self.forced_init.insert(sv, vec![self.obs_ptr+1+i]);
                                        }
                                    }
                                    self.obs_ptr += nb_missing;
                                    if self.obs_ptr == sv_nb_obs { // vehicule completion
                                        result = self.vehicule_completion(&result);
                                    }

                                    if nb_missing > 0 {
                                        continue 
                                    }
                                }
                            }
                        }
                    }
                    l
                },
                None => { 
                    // we're done iterating
                    // but that might be an early empty line.
                    // it is an early empty line if we were previously busy
                    //   and expecting <= 5 remaining fields,
                    if self.state == State::Body { // previously active
                        if self.obs_ptr > 0 { // previously active
                            // identify current Sv
                            let sv_size = 3;
                            let epoch_size = 32;
                            let vehicule_offset = self.vehicule_ptr * sv_size;
                            let min = epoch_size + vehicule_offset;
                            let max = min + sv_size;
                            let vehicule = &self.epoch_descriptor[min..max];
                            if let Ok(sv) = Sv::from_str(vehicule.trim()) {
                                // nb of obs for this constellation
                                let sv_nb_obs = obs_codes[&sv.constellation].len();
                                if sv_nb_obs - self.obs_ptr < 5 {
                                    let nb_missing = sv_nb_obs - self.obs_ptr;
                                    println!("Missing {} last field(s)", nb_missing);
                                    for i in 0..nb_missing { 
                                        self.flags_descriptor.push_str("  "); // both missing
                                        //schedule re/init
                                        if let Some(indexes) = self.forced_init.get_mut(&sv) {
                                            indexes.push(self.obs_ptr+1+i);
                                        } else {
                                            self.forced_init.insert(sv, vec![self.obs_ptr+1+i]);
                                        }
                                        self.obs_ptr += nb_missing;
                                        if self.obs_ptr >= sv_nb_obs { // vehicule completion
                                            result = self.vehicule_completion(&result);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    break 
                },
            };
            /*
                    // end of .lines() browsing
                    // or possible early empty line
                    if processed_size < total_size { // got some leftovers
                        if self.obs_ptr > 0 { // we were expecting data
                            // identify current Sv
                            let sv_size = 3;
                            let epoch_size = 32;
                            let vehicule_offset = self.vehicule_ptr * sv_size;
                            let min = epoch_size + vehicule_offset;
                            let max = min + sv_size;
                            let vehicule = &self.epoch_descriptor[min..max];
                            if let Ok(sv) = Sv::from_str(vehicule.trim()) {
                                // nb of obs for this constellation
                                let sv_nb_obs = obs_codes[&sv.constellation].len();
                                let nb_missing = std::cmp::min(5, sv_nb_obs - self.obs_ptr);
                                //DEBUG
                                println!("Early empty line - missing {} field(s)", nb_missing);
                                for i in 0..nb_missing { 
                                    self.flags_descriptor.push_str("  "); // both missing
                                    //schedule re/init
                                    if let Some(indexes) = self.forced_init.get_mut(&sv) {
                                        indexes.push(self.obs_ptr+1+i);
                                    } else {
                                        self.forced_init.insert(sv, vec![self.obs_ptr+1+i]);
                                    }
                                }
                                self.obs_ptr += nb_missing;
                                if self.obs_ptr == sv_nb_obs { // vehicule completion
                                    result = self.vehicule_completion(&result);
                                }
                            }
                        }
                    }
                    break ;
                }
            };*/
            
            //DEBUG
            println!("Working from LINE : \"{}\"", line);
            
            // [0] : COMMENTS (special case)
            if is_comment!(line) {
                if line.contains("RINEX FILE SPLICE") {
                    // [0*] SPLICE special comments
                    //      merged RINEX Files
                    self.state.reset();
                    //self.pointer = 0
                }
                result // feed content as is
                    .push_str(line);
                result // \n dropped by .lines()
                    .push_str("\n");
                continue
            }

            match self.state {
                State::EpochDescriptor => {
                    if self.epoch_ptr == 0 { // 1st line
                        // identify #systems
                        if line.len() > 32+3 { // at least 1 vehicule
                           self.epoch_ptr += 1;
                            let nb = &line[30..32];
                            if let Ok(u) = u16::from_str_radix(nb.trim(), 10) {
                                self.nb_vehicules = u.into();
                                println!("Identified {} vehicules", self.nb_vehicules);
                            } else {
                                return Err(Error::MalformedEpochDescriptor);
                            }
                        } else {
                            continue
                        }
                    } else {
                        self.epoch_ptr += 1;
                    }
                    self.epoch_descriptor.push_str(line);
                    self.epoch_descriptor.push_str("\n");

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
                    
                    let nb_lines = num_integer::div_ceil(self.nb_vehicules, 12) as u8;
                    if self.epoch_ptr == nb_lines { // end of descriptor
                        // format to CRINEX
                        self.epoch_descriptor = format_epoch_descriptor(&self.epoch_descriptor);
                        if self.first_epoch {
                            println!("INIT EPOCH with \"{}\"", self.epoch_descriptor);
                            self.epoch_diff.init(&self.epoch_descriptor);
                            result.push_str(&self.epoch_descriptor);
                            /////////////////////////////////////
                            //TODO
                            //missing clock offset field here
                            //next line should not always be empty
                            /////////////////////////////////////
                            result.push_str("\n");
                            self.first_epoch = false;
                        } else {
                            result.push_str(
                                &self.epoch_diff.compress(&self.epoch_descriptor));
                            result.push_str("\n");
                            /////////////////////////////////////
                            //TODO
                            //missing clock offset field here
                            //next line should not always be empty
                            /////////////////////////////////////
                            result.push_str("\n");
                        }

                        self.obs_ptr = 0;
                        self.vehicule_ptr = 0;
                        self.flags_descriptor.clear();
                        self.state = State::Body ;
                    }
                },
                State::Body => {
                    // identify current satellite using stored epoch description
                    let sv_size = 3;
                    let epoch_size = 32;
                    let vehicule_offset = self.vehicule_ptr * sv_size ;
                    let min = epoch_size + vehicule_offset; // epoch+ptr
                    let max = min + sv_size;
                    let vehicule = &self.epoch_descriptor[min..max];
                    println!("VEHICULE \"{}\"", vehicule);
                    if let Ok(sv) = Sv::from_str(vehicule.trim()) {
                        // nb of obs for this constellation
                        let sv_nb_obs = obs_codes[&sv.constellation].len();
                        // nb of obs in this line
                        let nb_obs_line = num_integer::div_ceil(line.len(), 17);
                        if self.obs_ptr > sv_nb_obs { // unexpected overflow
                            return Err(Error::MalformedEpochBody) // too many observables were found
                        }

                        // compress all observables
                        // and store flags for line completion
                        let mut observables = line.clone();
                        for _ in 0..nb_obs_line {
                            let index = std::cmp::min(16, observables.len()); // avoid overflow
                                                            // as some data flags might be omitted
                            let (data, rem) = observables.split_at(index);
                            let (obsdata, flags) = data.split_at(14);
                            observables = rem.clone();
                            if let Ok(obsdata) = f64::from_str(obsdata.trim()) {
                                let obsdata = f64::round(obsdata*1000.0) as i64; 
                                if flags.len() < 1 { // Both Flags ommited
                                    //DEBUG
                                    println!("OBS \"{}\" LLI \"X\" SSI \"X\"", obsdata);
                                    // data compression
                                    if let Some(sv_diffs) = self.sv_diff.get_mut(&sv) {
                                        // retrieve observable state
                                        if let Some(diffs) = sv_diffs.get_mut(self.obs_ptr) {
                                            let compressed :i64;
                                            // forced re/init is pending
                                            if let Some(indexes) = self.forced_init.get_mut(&sv) {
                                                if indexes.contains(&self.obs_ptr) {
                                                    // forced reinitialization
                                                    compressed = obsdata;
                                                    diffs.0.init(3, obsdata)
                                                        .unwrap();
                                                    // remove pending init,
                                                    // so we do not force reinitizalition more than once
                                                    for i in 0..indexes.len() {
                                                        if indexes[i] == self.obs_ptr {
                                                            indexes.remove(i);
                                                            break
                                                        }
                                                    }
                                                } else {
                                                    // compress data
                                                    compressed = diffs.0.compress(obsdata);
                                                }
                                            } else {
                                                // compress data
                                                compressed = diffs.0.compress(obsdata);
                                            }
                                            result.push_str(&format!("{} ", compressed));//append obs
                                        } else {
                                            // first time dealing with this observable
                                            let mut diff: (NumDiff, TextDiff, TextDiff) = (
                                                NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)?,
                                                TextDiff::new(),
                                                TextDiff::new(),
                                            );
                                            //DEBUG
                                            println!("INIT KERNELS with {} BLANK BLANK", obsdata);
                                            diff.0.init(3, obsdata)
                                                .unwrap();
                                            result.push_str(&format!("3&{} ", obsdata));//append obs
                                            diff.1.init(" "); // BLANK
                                            diff.2.init(" "); // BLANK
                                            self.flags_descriptor.push_str("  ");
                                            sv_diffs.push(diff);
                                        }
                                    } else {
                                        // first time dealing with this vehicule
                                        let mut diff: (NumDiff, TextDiff, TextDiff) = (
                                            NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)?,
                                            TextDiff::new(),
                                            TextDiff::new(),
                                        );
                                        //DEBUG
                                        println!("INIT KERNELS with {} BLANK BLANK", obsdata);
                                        diff.0.init(5, obsdata)
                                            .unwrap();
                                        diff.1.init("&"); // BLANK
                                        diff.2.init("&"); // BLANK
                                        self.flags_descriptor.push_str("  ");
                                        self.sv_diff.insert(sv, vec![diff]);
                                    }
                                } else { //flags.len() >=1 : Not all Flags ommited
                                    let (lli, ssi) = flags.split_at(1);
                                    println!("OBS \"{}\" - LLI \"{}\" - SSI \"{}\"", obsdata, lli, ssi);
                                    if let Some(sv_diffs) = self.sv_diff.get_mut(&sv) {
                                        // retrieve observable state
                                        if let Some(diffs) = sv_diffs.get_mut(self.obs_ptr) {
                                            // compress data
                                            let compressed :i64;
                                            // forced re/init is pending
                                            if let Some(indexes) = self.forced_init.get_mut(&sv) {
                                                if indexes.contains(&self.obs_ptr) {
                                                    // forced reinitialization
                                                    compressed = obsdata;
                                                    diffs.0.init(3, obsdata)
                                                        .unwrap();
                                                    // remove pending init,
                                                    // so we do not force reinitizalition more than once
                                                    for i in 0..indexes.len() {
                                                        if indexes[i] == self.obs_ptr {
                                                            indexes.remove(i);
                                                            break
                                                        }
                                                    }
                                                } else {
                                                    compressed = diffs.0.compress(obsdata);
                                                }
                                            } else {
                                                compressed = diffs.0.compress(obsdata);
                                            }
                                            result.push_str(&format!("{} ", compressed));
                                            let lli = diffs.1.compress(lli);
                                            self.flags_descriptor.push_str(&lli);
                                            if ssi.len() > 0 {
                                                let ssi = diffs.2.compress(ssi);
                                                self.flags_descriptor.push_str(&ssi);
                                            }

                                        } else {
                                            // first time dealing with this observable
                                            let mut diff: (NumDiff, TextDiff, TextDiff) = (
                                                NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)?,
                                                TextDiff::new(),
                                                TextDiff::new(),
                                            );
                                            diff.0.init(3, obsdata)
                                                .unwrap();
                                            result.push_str(&format!("3&{} ", obsdata));//append obs
                                            diff.1.init(lli);
                                            self.flags_descriptor.push_str(lli);
                                            if ssi.len() > 0 {
                                                //DEBUG
                                                println!("INIT KERNELS with {} - \"{}\" -  \"{}\"", obsdata, lli, ssi);
                                                diff.2.init(ssi);
                                                self.flags_descriptor.push_str(ssi);
                                            } else { // SSI omitted
                                                //DEBUG
                                                println!("INIT KERNELS with {} - \"{}\" - BLANK", obsdata, lli);
                                                diff.2.init("&"); // BLANK
                                                self.flags_descriptor.push_str(" ");
                                            }
                                            sv_diffs.push(diff);
                                        }
                                    } else {
                                        // first time dealing with this vehicule
                                        let mut diff: (NumDiff, TextDiff, TextDiff) = (
                                            NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)?,
                                            TextDiff::new(),
                                            TextDiff::new(),
                                        );
                                        diff.0.init(3, obsdata)
                                            .unwrap();
                                        result.push_str(&format!("3&{} ", obsdata));//append obs
                                        diff.1.init(lli); // BLANK
                                        self.flags_descriptor.push_str(lli);
                                        if ssi.len() > 0 {
                                            //DEBUG
                                            println!("INIT KERNELS with {} - \"{}\" -  \"{}\"", obsdata, lli, ssi);
                                            diff.2.init(ssi);
                                            self.flags_descriptor.push_str(ssi);
                                        } else { // SSI omitted
                                            //DEBUG
                                            println!("INIT KERNELS with {} - \"{}\" - BLANK", obsdata, lli);
                                            diff.2.init("&"); // BLANK
                                            self.flags_descriptor.push_str(" ");
                                        }
                                        self.sv_diff.insert(sv, vec![diff]);
                                    }
                                }
                            } else { //obsdata::f64::from_str()
                                // when the floating point observable parsing is in failure,
                                // we assume field is omitted
                                self.flags_descriptor.push_str("  ");
                                // schedule re/init 
                                if let Some(indexes) = self.forced_init.get_mut(&sv) {
                                    indexes.push(self.obs_ptr);
                                } else {
                                    self.forced_init.insert(sv, vec![self.obs_ptr]);
                                }
                            }
                            //DEBUG
                            println!("OBS {}/{}", self.obs_ptr+1, sv_nb_obs); 
                            self.obs_ptr += 1
                        } //for i..nb_obs in this line

                        if self.obs_ptr == sv_nb_obs { // vehicule completion
                            result = self.vehicule_completion(&result);
                        }
                    } else { // sv::from_str()
                        // failed to identify which vehicule we're dealing with
                        return Err(Error::VehiculeIdentificationError)
                    }
                },
            }//match(state)
        }//main loop
        Ok(result)
    }
    //notes:
    //si le flag est absent: "&" pour insérer un espace
    //tous les flags sont foutus a la fin en guise de dernier mot
}
