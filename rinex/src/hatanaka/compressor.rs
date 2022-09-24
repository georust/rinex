//! RINEX compression module
use crate::sv;
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
    /// vehicules counter in next body
    nb_vehicules: u16,
    /// vehicule pointer
    vehicule_ptr: u16,
    /// obs pointer
    obs_ptr: u8,
    /// Epoch differentiator 
    epoch_diff: TextDiff,
    /// Clock offset differentiator
    clock_diff: NumDiff,
    /// Vehicule differentiators
    sv_diff: HashMap<sv::Sv, Vec<(NumDiff, TextDiff, TextDiff)>>,
}
    
/// Reworks given content to match RINEX specifications
/// of an epoch descriptor
fn format_epoch_descriptor (content: &str, clock_offset: Option<i64>) -> Result<String, Error> {
    let mut result = String::new();
    //DEBUG
    println!("REWORKING \"{}\"", content);
    for line in content.lines() {
        result.push_str(line.trim());
    }
    result.push_str("\n");
    if let Some(offset) = clock_offset {
        //TODO
    } else {
        result.push_str("\n");
    }
    Ok(result)
}

impl Compressor {
    pub const MAX_COMPRESSION_ORDER: usize = NumDiff::MAX_COMPRESSION_ORDER;
    /// Creates a new compression structure 
    pub fn new (max_order: usize) -> Result<Self, Error> {
        Ok(Self {
            first_epoch: true,
            epoch_ptr: 0,
            epoch_descriptor: String::new(),
            state: State::default(),
            nb_vehicules: 0,
            vehicule_ptr: 0,
            obs_ptr: 0,
            epoch_diff: TextDiff::new(),
            clock_diff: NumDiff::new(max_order)?,
            sv_diff: HashMap::new(), // init. later
        })
    }
    
    /// Compresses given RINEX data to CRINEX 
    pub fn compress (&mut self, header: &header::Header, content: &str) -> Result<String, Error> {
        // Context sanity checks
        if header.rinex_type != Type::ObservationData {
            return Err(Error::NotObsRinexData) ;
        }
        
        // grab useful information for later
        let rnx_version = &header.version;
        let obs = header.obs
            .as_ref()
            .unwrap();
        let obs_codes = &obs.codes; 
        let crinex = obs.crinex
            .as_ref()
            .unwrap();
        let crx_version = crinex.version;
        
        let mut result : String = String::new();
        let mut lines = content.lines();
   
        loop {
            let line: &str = match lines.next() {
                Some(l) => l,
                None => break ,
            };
            
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
                        if line.len() < 32+3 { // at least 1 vehicule
                            return Err(Error::FaultyEpochDescriptor);
                        } else {
                            let nb = &line[30..32];
                            if let Ok(u) = u16::from_str_radix(nb.trim(), 10) {
                                self.nb_vehicules = u;
                                println!("Identified {} vehicules", self.nb_vehicules);
                            } else {
                                return Err(Error::FaultyEpochDescriptor);
                            }
                        }
                    }
                    self.epoch_ptr += 1;
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
                        // unwrap content to a single line
                        if self.first_epoch {
                            self.epoch_descriptor.replace_range(0..1, "&");
                        }
                        if let Ok(mut formatted) = format_epoch_descriptor(&self.epoch_descriptor, None) {
                            result.push_str(&formatted);
                            if self.first_epoch {
                                // initiate differentiator on unwrapped content
                                self.epoch_diff.init(&formatted);//TODO verifier que "&" ne pose pas de soucis
                                self.first_epoch = false; // never to reinitiate
                            }
                        } else {
                            return Err(Error::EpochReworkFailure);
                        }

                        self.state = State::Body ;
                    }
                },
                State::Body => {
                    // nb of observations for this constellation
                    /*let nb_total_obs = obs_codes[sv.constellation].len();
                    let nb_obs = line.len()/14; //TODO, et depend de RINX revision. Attention à l'arrondi
                    self.obs_ptr += nb_obs.into();
                    result.push_str("\n"); // TODO
                        // need to compress and wrapp all observables
                        // need to append and conclude line with lli/ssi flags
                    if obs_ptr == nb_obs { // last observation
                        // moving to next vehicule
                        self.vehicule_ptr += 1;
                        if self.vehicule_ptr == self.nb_vehicules { // last vehicule
                            self.state = State::EpochDescriptor 
                        }
                    }// else {
                    //    return Err(Error::TooManyObservationsInsideEpoch)
                    //}*/
                },
                _ => {}, // State::ClockOffsetDescriptor never ocurrs when compressing 
            }//match(state)

        }//main loop
        Ok(result)
    }
    //notes:
    //si le flag est absent: "&" pour insérer un espace
    //tous les flags sont foutus a la fin en guise de dernier mot
}
