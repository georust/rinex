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

mod fsm;
use fsm::Fsm;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("this is not a crinex file")]
    NotACrinexError,
    #[error("this is not an observation file")]
    NotObsRinexData,
    #[error("non supported crinex revision")]
    NonSupportedCrinexRevision,
    #[error("CRINEX1 standard mismatch")]
    FaultyCrinex1Format,
    #[error("CRINEX3 standard mismatch")]
    FaultyCrinex3Format,
    #[error("failed to identify sat. vehicule")]
    SvError(#[from] sv::Error),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("data recovery failed")]
    KernelError(#[from] kernel::Error),
    #[error("data recovery error")]
    DataRecoveryError,
}

/// Structure to compress / decompress RINEX data
pub struct Hatanaka {
    /// True only for first epoch to be processed 
    first_epoch: bool,
    /// finite state machine
    fsm: Fsm,
    /// True if header section is following 
    is_epoch_descriptor: bool,
    /// True if clock_offset line is following
    clock_offset: bool,
    /// line pointer
    pointer: u16,
    /// Epoch kernel
    epo_krn: Kernel,
    /// Clock offset kernel
    clk_krn: Kernel,
    /// Vehicule kernels
    sv_krn: HashMap<sv::Sv, Vec<(Kernel, Kernel, Kernel)>>,
}

impl Hatanaka {
    /// Creates a new compression / decompression tool
    pub fn new (max_order: usize) -> Hatanaka {
        Hatanaka {
            first_epoch: true,
            fsm: Fsm::default(),
            is_epoch_descriptor: true,
            clock_offset: false,
            pointer: 0,
            epo_krn: Kernel::new(0), // text
            clk_krn: Kernel::new(max_order), // numerical
            sv_krn: HashMap::new(), // init. later
        }
    }
    /// Decompresses (recovers) RINEX from given CRINEX record block.   
    /// This method will decompress and manage CRINEX comments or weird events properly.    
    /// This method will crash on header data: header section should be previously / separately parsed.    
    /// `header` : previously identified RINEX `header` section
    /// `content`: string reference extracted from a CRINEX record.    
    ///           This method is very convenient because `content` can have any shape you can think of.    
    ///           You can feed single CRINEX epoch lines, one at a time, just make sure it's terminated with an \n     
    ///           You can pass epochs one at a time (line groupings, several lines at once)    
    ///           You can also pass an entire CRINEX record at once      
    ///           You can even pass unevenly grouped chunks of epochs, even with COMMENTS unevenly inserted.    
    ///           Just make sure that `content` is never an empty string (at least 1 character),
    ///           which easily happens if you iterate through a CRINEX record with .lines() for instance.   
    ///           And also make sure to follow a standard CRINEX structure, which should always be the case   
    ///           if you iterate over a valid CRINEX.
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
        loop {
            // consume lines, if many were fed
            let line: &str = match lines.next() {
                Some(l) => l,
                None => break,
            };
            // [0] : COMMENTS (special case)
            if is_comment!(line) {
                if line.contains("RINEX FILE SPLICE") {
                    // [0*] SPLICE special comments
                    //      merged RINEX Files
                    //  --> reset FSM
                    self.is_epoch_descriptor = true;
                    self.pointer = 0
                }
                result.push_str(line); // feed content as is
                result.push_str("\n"); // dropped by .lines()
                continue
            }
            // [0*]: special epoch events
            //       with uncompressed descriptor
            if line.starts_with("> ") && !self.first_epoch {
                result.push_str(line); // feed as is..
                result.push_str("\n"); // dropped by .lines()
                continue
            }
            // [1] recover epoch descriptor 
            if self.header {
                self.recover_epoch_descriptor(crx_version.major, &line)?; 
                self.header = false;
                self.clock_offset = true;
                continue
            };
            // [2] recover clock offset, if any
            if self.clock_offset {
                clock_offset = match line.contains("&") {
                    false => {
                        if let Ok(num) = i64::from_str_radix(line.trim(),10) {
                            Some(num)
                        } else {
                            None // parsing fails on empty line
                        }
                    },
                    true => {
                        // kernel (re)init
                        let (n, rem) = line.split_at(1);
                        let n = u8::from_str_radix(n, 10)?;
                        let (_, num) = rem.split_at(1);
                        let num = i64::from_str_radix(num, 10)?;
                        self.clk_krn.init(
                            n.into(), 
                            Dtype::Numerical(num))
                            .unwrap();
                        Some(num)
                    },
                };
                // we can now fully recover the epoch description 
                let recovered_epoch =  // trick to recover textdiff
                    self.epo_krn.recover(Dtype::Text(String::from(" ")))?
                    .as_text()
                    .unwrap();
                let recovered_epoch = recovered_epoch.as_str().trim_end();
                match rnx_version.major {
                    1|2 => { // old RINEX
                        // system # id is appended
                        // and wrapped on as many lines as needed
                        let (epoch, systems) = recovered_epoch.split_at(32);
                        result.push_str(epoch);
                        let mut begin = 0;
                        // terminate first line with required content
                        let end = std::cmp::min(begin+12*3, systems.len());
                        result.push_str(&systems[begin..end]);
                        // squeeze clock offset here, if any
                        if let Some(offset) = clock_offset {
                            result.push_str(&format!("  {:3.9}", (offset as f64)/1000.0_f64))
                        }
                        loop { // missing lines to fit remaining systems 
                            begin += 12*3; // `systems` pointer
                            if begin >= systems.len() {
                                break
                            }
                            let end = std::cmp::min(begin+12*3, systems.len());
                            result.push_str("\n                                ");
                            result.push_str(&systems[begin..end]);
                        }
                    },
                    _ => { // modern RINEX
                        result.push_str(recovered_epoch.split_at(35).0);
                        if let Some(offset) = clock_offset {
                            result.push_str(&format!("         {:3.12}", (offset as f64)/1000.0_f64))
                        }
                    }
                };
                result.push_str("\n");
                self.clock_offset = false;
                continue
            }
            // [3] inside epoch content
            let recovered_epoch =  // trick to recover textdiff
                self.epo_krn.recover(Dtype::Text(String::from(" ")))?
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
                _ => return Err(Error::NonSupportedCrinexRevision)
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
            let mut obs_count : usize = 0;
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
                                //TODO also strict RINEX3 please
                                if (i+1).rem_euclid(5) == 0 { // maximal nb of OBS per line
                                    result.push_str("\n")
                                }
                            }
                        } else {
                            result.push_str("              "); // BLANK data
                            result.push_str(" "); // BLANK lli
                            result.push_str(" "); // BLANK ssi
                            if rnx_version.major < 3 { // old RINEX
                                //TODO and also on strict RINEX3 compatibility please
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
                                    //TODO and also on strict RINEX3 compatibility please
                                    if (i+1).rem_euclid(5) == 0 { // maximal nb of OBS per line
                                        result.push_str("\n")
                                    }
                                }
                            } else {
                                result.push_str("              "); // BLANK data
                                result.push_str(" "); // BLANK lli
                                result.push_str(" "); // BLANK ssi
                                if rnx_version.major < 3 { // old RINEX
                                    //TODO and also on strict RINEX3 compatibility please
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
                self.is_epoch_descriptor = true // reset FSM
            }
        }

        Ok(result)
    }
    /// Recovers epoch descriptor from given content
    fn recover_epoch_descriptor (&mut self, revision_major: u8, line: &str) -> Result<String, Error> {
        // CRINEX sanity checks
        if self.first_epoch {
            match revision_major {
                1 => {
                    if !line.starts_with("&") {
                        return Err(Error::FaultyCrinex1Format)
                    }
                },
                3 => {
                    if !line.starts_with("> ") {
                        return Err(Error::FaultyCrinex3Format)
                    }
                },
                _ => return Err(Error::NonSupportedCrinexRevision),
            }
        }
        if self.first_epoch {
            self.epo_krn.init( // init this kernel
                0, // is always a textdiff
                Dtype::Text(line.to_string()))?;
            self.first_epoch = false
        }
        Ok(self.epo_krn.recover(Dtype::Text(line.to_string()))?
            .as_text()
            .unwrap())
    }
}
