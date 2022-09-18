//! RINEX compression / decompression module
use crate::sv;
use crate::header;
use crate::is_comment;
use crate::types::Type;
use thiserror::Error;
use std::str::FromStr;
use std::collections::HashMap;

mod kernel;
use kernel::{Kernel, Dtype};

pub enum State {
    EpochDescriptor,
    ClockOffsetDescriptor,
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

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("This is not a CRX file")]
    NotACrinexError,
    #[error("This is not an Observation file")]
    NotObsRinexData,
    #[error("Non supported CRX revision")]
    NonSupportedCrxVersion,
    #[error("First epoch not delimited by \"&\"")]
    FaultyCrx1FirstEpoch,
    #[error("First epoch not delimited by \">\"")]
    FaultyCrx3FirstEpoch,
    #[error("Failed to initialize epoch kernel")]
    EpochKernelInitError,
    #[error("Failed to parse clock offset init order")]
    ClockOffsetOrderError,
    #[error("Failed to parse clock offset value")]
    ClockOffsetValueError,
    #[error("Failed to init clock offset kernel")]
    ClockOffsetInitError,
    #[error("Failed to recover clock offset")]
    ClockOffsetRecoveryFailure,
    #[error("Failed to recover epoch description")]
    EpochRecoveryFailure,
    #[error("Recovered epoch content seems faulty")]
    FaultyRecoveredEpoch,
    #[error("Failed to rework epoch to match standards")]
    EpochReworkFailure,
    #[error("Failed to latch new epoch description")]
    EpochLatchError,
    #[error("failed to identify sat. vehicule")]
    SvError(#[from] sv::Error),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("data recovery failed")]
    KernelError(#[from] kernel::Error),
}

/// Structure to compress / decompress RINEX data
pub struct Hatanaka {
    /// finite state machine
    state: State,
    /// True only for first epoch ever processed 
    first_epoch: bool,
    /// line pointer
    pointer: u16,
    /// Epoch kernel
    epoch_krn: Kernel,
    /// Clock offset kernel
    clk_krn: Kernel,
    /// Vehicule kernels
    sv_krn: HashMap<sv::Sv, Vec<(Kernel, Kernel, Kernel)>>,
}
    
/// Reworks given content to match RINEX specifications
/// of an epoch descriptor
fn format_epoch (version: u8, content: &str, clock_offset: Option<i64>) -> Result<String, Error> {
    let mut result = String::new();
    //DEBUG
    //println!("REWORKING \"{}\"", content);
    match version {
        1|2 => { // old RINEX
            // append Systems #ID,
            //  on as many lines as needed
            let min_size = 32 + 3; // epoch descriptor
                // +XX+XYY at least one vehicule
            if content.len() < min_size { // parsing would fail
                return Err(Error::FaultyRecoveredEpoch)
            }
            
            let (epoch, systems) = content.split_at(32); // grab epoch
            result.push_str(&epoch.replace("&", " ")); // rework
            
            //CRINEX has systems squashed in a single line
            // we just split it to match standard definitions
            // .. and don't forget the tab
            for i in 0..num_integer::div_ceil(systems.len(), 36) {
                if i == 0 {
                    // squeeze clock offset here, if any
                    if let Some(value) = clock_offset {
                        result.push_str(&format!("  {:3.9}", (value as f64)/1000.0_f64))
                    }
                } else { // tab indent
                    // TODO: improve please
                    result.push_str("\n                                ");
                }

                let max_offset = std::cmp::min( // avoids overflowing
                    (i+1)*12*3,
                    systems.len(),
                );
                result.push_str(&systems[
                    i*12*3 .. max_offset]);
            }
        },
        _ => { // Modern RINEX case
            // Systems #ID to be passed on future lines
            if content.len() < 35 { // parsing would fail
                return Err(Error::FaultyRecoveredEpoch)
            }
            let (epoch, _) = content.split_at(35); 
            result.push_str(epoch);
            
            //TODO clock offset
            if let Some(value) = clock_offset {
                result.push_str(&format!("         {:3.12}", (value as f64)/1000.0_f64))
            }
        },
    }
    result.push_str("\n"); // corret format
    Ok(result)
}

impl Hatanaka {
    /// Creates a new compression / decompression tool
    pub fn new (max_order: usize) -> Hatanaka {
        Hatanaka {
            first_epoch: true,
            state: State::default(),
            pointer: 0,
            epoch_krn: Kernel::new(0), // text
            clk_krn: Kernel::new(max_order), // numerical
            sv_krn: HashMap::new(), // init. later
        }
    }
    /// Decompresses (recovers) RINEX from given CRINEX content.
    /// Content can be header / comments
    /// they will be passed as is, as expected.
    /// Content can be entire CRINEX epochs, group of lines, or a single line at a time. Content must be at least '\n', pay attention
    /// to empty clock offset lines.
    /// `header`: previously identified RINEX `header` section
    /// `content`: CRINEX record content
    /// `result`: returns decompressed (recovered) block from provided block content
    pub fn decompress (&mut self, header: &header::Header, content : &str) -> Result<String, Error> {
        // Context sanity checks
        if !header.is_crinex() {
            return Err(Error::NotACrinexError)
        }
        if header.rinex_type != Type::ObservationData {
            return Err(Error::NotObsRinexData)
        }
        // grab useful information for later
        let rnx_version = &header.version;
        let obs = header.obs
            .as_ref()
            .unwrap();
        let crinex = obs.crinex
            .as_ref()
            .unwrap();
        let crx_version = crinex.version;
        let obs_codes = &obs.codes; 
        
        // pre defined maximal compression order
        //  ===> to adapt all other kernels accordingly
        let m = self.clk_krn
            .state
            .len()-1; 
        let mut result : String = String::new();
        let mut lines = content.lines();
        let mut clock_offset : Option<i64> = None;
        let mut obs_count : usize = 0;

        loop {
            let line: &str = match lines.next() {
                Some(l) => l,
                None => break,
            };

            //DEBUG
            //println!("Working from LINE : \"{}\"", line);
            
            // [0] : COMMENTS (special case)
            if is_comment!(line) {
                if line.contains("RINEX FILE SPLICE") {
                    // [0*] SPLICE special comments
                    //      merged RINEX Files
                    self.state.reset();
                    self.pointer = 0
                }
                result // feed content as is
                    .push_str(line);
                result // \n dropped by .lines()
                    .push_str("\n");
                continue
            }

            // [0*]: special epoch events
            //       with uncompressed descriptor
            //       (CRNX3)
            if line.starts_with("> ") && !self.first_epoch {
                result // feed content as is
                    .push_str(line);
                result // \n dropped by .lines()
                    .push_str("\n");
                continue
            }
            
            match self.state {
                State::EpochDescriptor => {
                    if self.first_epoch {
                        if crx_version.major == 1 {
                            if !line.starts_with("&") {
                                return Err(Error::FaultyCrx1FirstEpoch) ;
                            }
                        }
                        if crx_version.major == 3 {
                            if !line.starts_with(">") {
                                return Err(Error::FaultyCrx3FirstEpoch) ;
                            }
                        }
                        //DEBUG
                        //println!("Initializing EPOCH KERNEL with");
                        //println!("\"{}\"", line);
                        
                        // Kernel initialization,
                        // only once, always text based
                        if self.epoch_krn.init(
                            0, // textdiff
                            Dtype::Text(line.to_string())).is_err() {
                            return Err(Error::EpochKernelInitError) ;
                        }
                        self.first_epoch = false; //never to be re-initialized
                    
                    } else {
                        // here we use "recover" just to latch
                        // the new string to textdiff()
                        if self.epoch_krn.recover(
                            Dtype::Text(line.to_string())).is_err() {
                            return Err(Error::EpochLatchError) ;
                        }
                    }
                    
                    self.state = State::ClockOffsetDescriptor;
                }, // state::EpochDescriptor

                State::ClockOffsetDescriptor => {
                    if line.contains("&") {
                        // clock offset kernel (re)init
                        //   parse init. parameters
                        let (n, rem) = line.split_at(1);
                        if let Ok(order) = u8::from_str_radix(n, 10) {
                            let (_, value) = rem.split_at(1);
                            if let Ok(value) = i64::from_str_radix(value, 10) {
                                if self.clk_krn
                                    .init(
                                        order.into(),
                                        Dtype::Numerical(value)).is_err()
                                {
                                    return Err(Error::ClockOffsetInitError)
                                }
                            } else {
                                return Err(Error::ClockOffsetValueError)
                            }
                        } else {
                            return Err(Error::ClockOffsetOrderError)
                        }
                    } else { // --> nominal clock offset line
                        if let Ok(value) = i64::from_str_radix(line.trim(), 10) {
                            clock_offset = Some(value); // latch for later
                        } 
                    }

                    // Epoch Recovery
                    if let Ok(recover) = self
                        .epoch_krn // trick: epoch description was previously stored
                            // we're at clock offset description,
                            // here I take advantage of the textdiff behavior
                            .recover(Dtype::Text(String::from(" "))) {
                        if let Some(recovered) = recover.as_text() {
                            // now we must rebuild epoch description, according to standards
                            let recovered = &recovered.trim_end();
                            //TODO missing clock offset here please
                            if let Ok(epoch) = format_epoch(rnx_version.major, &recovered, clock_offset) {
                                //DEBUG
                                //println!("REWORKD   \"{}\"", epoch);
                                result.push_str(&epoch); 
                            } else {
                                return Err(Error::EpochReworkFailure) ;
                            }
                        }
                    } else {
                        return Err(Error::EpochRecoveryFailure) ;
                    }

                    self.state = State::Body ;
                }, // state::ClockOffsetDescriptor

                State::Body => {
                    // [3] inside epoch content
                    let recovered_epoch =  // trick to recover textdiff
                        self.epoch_krn.recover(Dtype::Text(String::from(" ")))?
                        .as_text()
                        .unwrap();
                    let epo = recovered_epoch.as_str().trim_end();
                    let mut offset : usize =
                        2    // Y
                        +2+1 // m
                        +2+1 // d
                        +2+1 // h
                        +2+1 // m
                        +11  // s
                        +1;  // ">" or "&" init marker
                    if rnx_version.major > 2 { offset += 2 } // Y is 4 digit
                    if epo.starts_with("> ") { offset += 1 } // CRINEX3 has 1 extra whitespace
                    let (_, rem) = epo.split_at(offset);
                    let (_, rem) = rem.split_at(3); // _ is epoch flag
                    let (n, _) = rem.split_at(3);
                    let nb_sv = u16::from_str_radix(n.trim(), 10)?;
                    //     ---> identify nb of satellite vehicules
                    //     ---> identify which system we're dealing with
                    //          using recovered header
                    let offset : usize = match crx_version.major {
                        1 => std::cmp::min((32 + 3*(self.pointer+1)).into(), epo.len()),
                        3 => std::cmp::min((41 + 3*(self.pointer+1)).into(), epo.len()),
                        _ => return Err(Error::NonSupportedCrxVersion)
                    };
                    let system = epo.split_at(offset.into()).0;
                    let system = system.split_at(system.len()-3).1; // last 3 XXX
                    if rnx_version.major > 2 {
                        result.push_str(&system.to_string()); // Modern rinex needs XXX on every line
                    }

                    let sv = sv::Sv::from_str(system)?;
                    let codes = &obs_codes[&sv.constellation];
                    if !self.sv_krn.contains_key(&sv) {
                        // first time dealing with this system
                        // add an entry for each obscode
                        let mut v : Vec<(Kernel,Kernel,Kernel)> = Vec::with_capacity(12);
                        for _ in codes {
                            let mut kernels = (
                                Kernel::new(m), // OBS
                                Kernel::new(0), // SSI
                                Kernel::new(0), // LLI
                            );
                            // init with BLANK 
                            kernels.1 // LLI
                                .init(0, Dtype::Text(String::from(" "))) // textdiff
                                .unwrap();
                            kernels.2 // SSI
                                .init(0, Dtype::Text(String::from(" "))) // textdiff
                                .unwrap();
                            v.push(kernels)
                        }
                        self.sv_krn.insert(sv, v); // creates new entry
                    }
                    
                    // try to grab all data,
                    // might fail in case it's truncated by compression
                    let mut rem = line.clone();
                    let mut obs_data : Vec<Option<i64>> = Vec::with_capacity(12);
                    loop {
                        if obs_count == codes.len() {
                            // FLAGS fields
                            //  ---> parse & run textdiff on each individual character
                            //   --> then format final output line
                            let mut obs_flags : Vec<String> = Vec::with_capacity(obs_data.len()*2);
                            // [+] grab all provided and apply textdiff
                            //     append BLANK in case not provided,
                            //     this approach produces 1 flag (either blank or provided/recovered) 
                            //     to previously provided/recovered OBS data
                            let obs = self.sv_krn.get_mut(&sv)
                                .unwrap();
                            for i in 0..rem.len() { // 1 character at a time
                                let flag = i%2;
                                if flag == 0 {
                                    obs_flags.push(
                                        obs[i/2] // two flags per OBS
                                            .1 // lli
                                            .recover(Dtype::Text(rem[i..i+1].to_string()))
                                            .unwrap()
                                                .as_text()
                                                .unwrap())
                                } else {
                                    obs_flags.push(
                                        obs[i/2] // two flags per OBS
                                            .2 // ssii
                                            .recover(Dtype::Text(rem[i..i+1].to_string()))
                                            .unwrap()
                                                .as_text()
                                                .unwrap())
                                }
                            }
                            for i in obs_flags.len()..obs_data.len()*2 {
                                // some flags were not provided
                                // meaning text is maintained
                                let flag = i%2;
                                if flag == 0 {
                                    obs_flags.push(
                                        obs[i/2]
                                            .1 // lli
                                            .recover(Dtype::Text(String::from(" ")))
                                            .unwrap()
                                                .as_text()
                                                .unwrap())
                                } else {
                                    obs_flags.push(
                                        obs[i/2]
                                            .2 // lli
                                            .recover(Dtype::Text(String::from(" ")))
                                            .unwrap()
                                                .as_text()
                                                .unwrap())
                                }
                            }
                            for i in 0..obs_data.len() {
                                if let Some(data) = obs_data[i] {
                                    // --> data field was found & recovered
                                    result.push_str(&format!(" {:13.3}", data as f64 /1000_f64)); // F14.3
                                    result.push_str(&obs_flags[i*2]); // lli
                                    result.push_str(&obs_flags[i*2+1]); // ssi
                                    if rnx_version.major < 3 { // old RINEX
                                        if (i+1).rem_euclid(5) == 0 { // maximal nb of OBS per line
                                            result.push_str("\n")
                                        }
                                    }
                                } else {
                                    result.push_str("              "); // BLANK data
                                    result.push_str(" "); // BLANK lli
                                    result.push_str(" "); // BLANK ssi
                                    if rnx_version.major < 3 { // old RINEX
                                        if (i+1).rem_euclid(5) == 0 { // maximal nb of OBS per line
                                            result.push_str("\n")
                                        }
                                    }
                                }
                            }
                            result.push_str("\n");
                            break
                        }
                        let next_wsp = match rem.find(' ') {
                            Some(l_off) => l_off+1,
                            None => { 
                                // line is either terminated by one last compressed code
                                // or empty content
                                // [+] try to parse one last obs 
                                if rem.contains("&") {
                                    // kernel (re)init 
                                    let index = rem.find("&").unwrap();
                                    let (order, rem) = rem.split_at(index);
                                    let order = u8::from_str_radix(order.trim(),10)?;
                                    let (_, data) = rem.split_at(1);
                                    let data = i64::from_str_radix(data.trim(), 10)?;
                                    let obs = self.sv_krn.get_mut(&sv)
                                        .unwrap();
                                    obs[obs_count]
                                        .0 // OBS
                                        .init(
                                            order.into(),
                                            Dtype::Numerical(data))
                                            .unwrap();
                                    obs_data.push(Some(data));
                                    obs_count += 1
                                } else {
                                    // regular compression
                                    if let Ok(num) = i64::from_str_radix(rem.trim(),10) {
                                        let obs = self.sv_krn.get_mut(&sv)
                                            .unwrap();
                                        let recovered = obs[obs_count]
                                            .0 // OBS
                                            .recover(
                                                Dtype::Numerical(num))
                                            .unwrap()
                                            .as_numerical()
                                            .unwrap();
                                        obs_data.push(Some(recovered))
                                    }
                                }
                                //  --> format this line correctly
                                for i in 0..obs_data.len() {
                                    if let Some(data) = obs_data[i] {
                                        // --> data field was found & recovered
                                        result.push_str(&format!(" {:13.3}", data as f64 /1000_f64)); // F14.3
                                        // ---> related flag content
                                        let obs = self.sv_krn.get_mut(&sv)
                                            .unwrap();
                                        let lli = obs[i]
                                            .1 // LLI
                                            .recover(Dtype::Text(String::from(" "))) // trick to recover
                                            .unwrap()
                                            .as_text()
                                            .unwrap();
                                        let ssi = obs[i]
                                            .2 // SSI
                                            .recover(Dtype::Text(String::from(" "))) // trick to recover
                                            .unwrap()
                                            .as_text()
                                            .unwrap();
                                        result.push_str(&lli); // FLAG
                                        result.push_str(&ssi); // FLAG 
                                        if rnx_version.major < 3 { // old RINEX
                                            if (i+1).rem_euclid(5) == 0 { // maximal nb of OBS per line
                                                result.push_str("\n")
                                            }
                                        }
                                    } else {
                                        result.push_str("              "); // BLANK data
                                        result.push_str(" "); // BLANK lli
                                        result.push_str(" "); // BLANK ssi
                                        if rnx_version.major < 3 { // old RINEX
                                            if (i+1).rem_euclid(5) == 0 { // maximal nb of OBS per line
                                                result.push_str("\n")
                                            }
                                        }
                                    }
                                }
                                result.push_str("\n");
                                break // EOL
                            },
                        };
                        let (roi, r) = rem.split_at(next_wsp);
                        rem = r;
                        if roi == " " { // BLANK field
                            obs_count += 1;
                            obs_data.push(None);
                            continue // compressed non existing obs 
                        }
                        let (init_order, data) : (Option<u16>, i64) = match roi.contains("&") {
                            false => {
                                (None, i64::from_str_radix(roi.trim(),10)?)
                            },
                            true => {
                                let (init_order, remainder) = roi.split_at(1);
                                let (_, data) = remainder.split_at(1);
                                (Some(u16::from_str_radix(init_order, 10)?),
                                i64::from_str_radix(data.trim(), 10)?)
                            },
                        };
                        if let Some(order) = init_order {
                            //(re)init that kernel
                            let obs = self.sv_krn.get_mut(&sv)
                                .unwrap();
                            obs[obs_count]
                                .0 // OBS
                                .init(
                                    order.into(),
                                    Dtype::Numerical(data))
                                    .unwrap();
                            obs_data.push(Some(data))
                        } else {
                            let obs = self.sv_krn.get_mut(&sv)
                                .unwrap();
                            let recovered = obs[obs_count]
                                .0 // OBS
                                .recover(Dtype::Numerical(data))
                                .unwrap();
                            let recovered = recovered
                                .as_numerical()
                                .unwrap();
                            obs_data.push(Some(recovered))
                        }
                        obs_count +=1
                    } // for all OBS
                    self.pointer += 1;
                    if self.pointer == nb_sv { // nothing else to parse
                        self.pointer = 0; // reset
                        self.state.reset();
                    }
                }, // state::Body
            } // match(state)
        }
        Ok(result)
    }
}
