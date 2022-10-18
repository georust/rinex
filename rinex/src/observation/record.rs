//! `ObservationData` parser and related methods
use thiserror::Error;
use std::str::FromStr;
//use chrono::Timelike;
use bitflags::bitflags;
use std::io::Write;
use std::collections::{BTreeMap, HashMap};

use crate::{
	sv, Sv,
	constellation, Constellation,
	Epoch, EpochFlag,
	epoch::{
		ParseDateError, str2date,
	},
	Header,
	version::Version,
	writer::BufferedWriter,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// `Ssi` describes signals strength
#[repr(u8)]
#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Ssi {
    /// Ssi ~= 0 dB/Hz
    DbHz0 = 0,
    /// Ssi < 12 dB/Hz
    DbHz12 = 1,
    /// 12 dB/Hz <= Ssi < 17 dB/Hz
    DbHz12_17 = 2, 
    /// 18 dB/Hz <= Ssi < 23 dB/Hz
    DbHz18_23 = 3, 
    /// 24 dB/Hz <= Ssi < 29 dB/Hz
    DbHz24_29 = 4, 
    /// 30 dB/Hz <= Ssi < 35 dB/Hz
    DbHz30_35 = 5, 
    /// 36 dB/Hz <= Ssi < 41 dB/Hz
    DbHz36_41 = 6, 
    /// 42 dB/Hz <= Ssi < 47 dB/Hz
    DbHz42_47 = 7, 
    /// 48 dB/Hz <= Ssi < 53 dB/Hz
    DbHz48_53 = 8, 
    /// Ssi >= 54 dB/Hz 
    DbHz54 = 9, 
}

impl std::fmt::Display for Ssi {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DbHz0 => "0".fmt(f),
            Self::DbHz12 => "1".fmt(f),
            Self::DbHz12_17 => "2".fmt(f),
            Self::DbHz18_23 => "3".fmt(f),
            Self::DbHz24_29 => "4".fmt(f),
            Self::DbHz30_35 => "5".fmt(f),
            Self::DbHz36_41 => "6".fmt(f),
            Self::DbHz42_47 => "7".fmt(f),
            Self::DbHz48_53 => "8".fmt(f),
            Self::DbHz54 => "9".fmt(f),
        }
    }
}

impl Default for Ssi {
    fn default() -> Ssi { Ssi::DbHz54 }
}

impl FromStr for Ssi {
    type Err = std::io::Error;
    fn from_str (code: &str) -> Result<Self, Self::Err> {
        match code {
            "0" => Ok(Ssi::DbHz0),
            "1" => Ok(Ssi::DbHz12),
            "2" => Ok(Ssi::DbHz12_17),
            "3" => Ok(Ssi::DbHz18_23),
            "4" => Ok(Ssi::DbHz24_29),
            "5" => Ok(Ssi::DbHz30_35),
            "6" => Ok(Ssi::DbHz36_41),
            "7" => Ok(Ssi::DbHz42_47),
            "8" => Ok(Ssi::DbHz48_53),
            "9" => Ok(Ssi::DbHz54),
            _ =>  Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid Ssi code")),
        }
    }
}

impl Ssi {
    /// Returns true if `self` is a bad signal level, very poor quality,
    /// measurements should be discarded
    pub fn is_bad (self) -> bool {
        self <= Ssi::DbHz18_23
    }
    /// Returns true if `self` is a weak signal level, poor quality
    pub fn is_weak (self) -> bool {
        self < Ssi::DbHz30_35
    }
    /// Returns true if `self` is a strong signal level, good quality as defined by standard
    pub fn is_strong (self) -> bool {
        self >= Ssi::DbHz30_35
    }
    /// Returns true if `self` is a very strong signal level, very high quality
    pub fn is_excellent (self) -> bool {
        self > Ssi::DbHz42_47
    }
    /// Returns true if `self` matches a strong signal level (defined by standard)
    pub fn is_ok (self) -> bool { self.is_strong() }
}

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct LliFlags: u8 {
        /// Current epoch is marked Ok or Unknown status 
        const OK_OR_UNKNOWN = 0x00;
        /// Lock lost between previous observation and current observation,
        /// cycle slip is possible
        const LOCK_LOSS = 0x01;
        /// Opposite wavelenght factor to the one defined
        /// for the satellite by a previous WAVELENGTH FACT comment,
        /// or opposite to default value, is not previous WAVELENFTH FACT comment
        const HALF_CYCLE_SLIP = 0x02;
        /// Observing under anti spoofing,
        /// might suffer from decreased SNR - decreased signal quality
        const UNDER_ANTI_SPOOFING = 0x04;
    }
}

#[derive(Copy, Clone, Debug)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ObservationData {
	/// physical measurement
	pub obs: f64,
	/// Lock loss indicator 
	pub lli: Option<LliFlags>,
	/// Signal strength indicator
	pub ssi: Option<Ssi>,
}

impl ObservationData {
	/// Builds new ObservationData structure from given predicates
    pub fn new (obs: f64, lli: Option<LliFlags>, ssi: Option<Ssi>) -> ObservationData {
		ObservationData {
			obs,
			lli,
			ssi,
		}
	}
    /// Returns `true` if self is determined as `ok`.    
    /// self is declared `ok` if LLI and SSI flags are not provided,
    /// because they are considered as unknown/ok if missing by default.   
    /// If LLI exists:    
    ///    + LLI must match the LliFlags::OkOrUnknown flag (strictly)    
    /// if SSI exists:    
    ///    + SSI must match the .is_ok() criteria, refer to API 
    pub fn is_ok (self) -> bool {
        let lli_ok = self.lli.unwrap_or(LliFlags::OK_OR_UNKNOWN) == LliFlags::OK_OR_UNKNOWN;
        let ssi_ok = self.ssi.unwrap_or(Ssi::default()).is_ok();
        lli_ok && ssi_ok
    }

    /// Returns Real Distance, by converting observed pseudo range,
    /// and compensating for distant and local clock offsets.
    /// See [p17-p18 of the RINEX specifications]. It makes only
    /// sense to apply this method on Pseudo Range observations.
    /// - rcvr_offset: receiver clock offset for this epoch, given in file
    /// - sv_offset: sv clock offset
    /// - bias: other (optionnal..) additive biases
    pub fn pr_real_distance (&self, rcvr_offset: f64, sv_offset: f64, biases: f64) -> f64 {
        self.obs + 299_792_458.0_f64 * (rcvr_offset - sv_offset) + biases
    }
}

/// Observation Record content. 
/// Measurements are sorted by [epoch::Epoch].
/// An epoch possibly comprises the receiver clock offset
/// and a list of physical measurements, sorted by Space vehicule and observable.
/// Example of Observation Data browsing and manipulation
/// ```
/// use rinex::*;
/// // grab a CRINEX (compressed OBS RINEX)
/// let rnx = Rinex::from_file("../test_resources/CRNX/V3/KUNZ00CZE.crx")
///    .unwrap();
/// // grab record
/// let record = rnx.record.as_obs()
///    .unwrap();
/// // browse epochs
/// for (epoch, (clock_offset, vehicules)) in record.iter() {
///    if let Some(clock_offset) = clock_offset {
///        // got clock offset @ given epoch
///    }
///    for (vehicule, observables) in vehicules.iter() {
///        for (observable, observation) in observables.iter() {
///            /// `observable` is a standard 3 letter string code
///            /// main measurement is `observation.data` (f64)
///            if let Some(lli) = observation.lli {
///                // Sometimes observations have an LLI flag attached to them,
///                // implemented in the form of a `bitflag`,
///                // for convenient binary masking
///
///            }
///            if let Some(ssii) = observation.ssi {
///                // Sometimes observations come with an SSI indicator
///            }
///        }
///    }
/// }
/// ```
pub type Record = BTreeMap<Epoch, 
    (Option<f64>, 
    BTreeMap<sv::Sv, HashMap<String, ObservationData>>)>;

#[derive(Error, Debug)]
/// OBS Data `Record` parsing specific errors
pub enum Error {
    #[error("failed to parse date")]
    ParseDateError(#[from] ParseDateError),
    #[error("failed to parse epoch flag")]
    ParseEpochFlagError(#[from] std::io::Error),
    #[error("failed to identify constellation")]
    ConstellationError(#[from] constellation::Error),
    #[error("failed to parse sv")]
    SvError(#[from] sv::Error),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse vehicules properly (n_sat mismatch)")]
    EpochParsingError,
	#[error("line is empty")]
	MissingData,
}

/// Returns true if given content matches a new OBSERVATION data epoch
pub fn is_new_epoch (line: &str, v: Version) -> bool {
    let parsed: Vec<&str> = line
        .split_ascii_whitespace()
        .collect();
	if v.major < 3 {
        // old RINEX
        // epoch block is type dependent
        if parsed.len() > 6 {
            //  * contains at least 6 items
            let mut datestr = parsed[0].to_owned(); // Y
            datestr.push_str(" ");
            datestr.push_str(parsed[1]); // m
            datestr.push_str(" ");
            datestr.push_str(parsed[2]); // d
            datestr.push_str(" ");
            datestr.push_str(parsed[3]); // h
            datestr.push_str(" ");
            datestr.push_str(parsed[4]); // m
            datestr.push_str(" ");
            datestr.push_str(parsed[5]); // s
            str2date(&datestr).is_ok()
        } else {
            false // does not match
                // an epoch descriptor
        }
    } else {
        // Modern RINEX
        // OBS::V3 behaves like all::V4
        match line.chars().nth(0) {
            Some(c) => {
                c == '>' // epochs always delimited
                    // by this new identifier
            },
            _ => false,
        }
    }
}

/// Builds `Record` entry for `ObservationData`
/// from given epoch content
pub fn parse_epoch (header: &Header, content: &str)
        -> Result<(Epoch, Option<f64>, BTreeMap<sv::Sv, HashMap<String, ObservationData>>), Error> 
{
    let mut lines = content.lines();
    let mut line = match lines.next() {
		Some(l) => l,
		_ => return Err(Error::MissingData),
	};

    // epoch::
    let mut offset : usize = 
        2+1 // Y
        +2+1 // d
        +2+1 // m
        +2+1 // h
        +2+1 // m
        +11; // secs
    
    // V > 2 epoch::year is a 4 digit number
    if header.version.major > 2 {
        offset += 2
    }

    // V > 2 might start with a ">" marker
    if line.starts_with(">") {
        line = line.split_at(1).1.clone();
    }

    let (date, rem) = line.split_at(offset);
    let (flag, rem) = rem.split_at(3);
    let (n_sat, rem) = rem.split_at(3);
    let n_sat = u16::from_str_radix(n_sat.trim(), 10)?;

    let flag = EpochFlag::from_str(flag.trim())?;
    let date = str2date(date)?; 
    let epoch = Epoch::new(date, flag);
	
    // previously identified observables (that we expect)
    let obs = header.obs
        .as_ref()
        .unwrap();
    let observables = &obs.codes;
    
    // grab possible clock offset 
    let offs : Option<&str> = match header.version.major < 2 {
        true => { // RINEX 2
            // clock offsets are last 12 characters
            if line.len() > 60-12 {
                Some(line.split_at(60-12).1.trim())
            } else {
                None
            }
        },
        false => { // RINEX 3
            let min_len : usize = 
                 4+1 // y
                +2+1 // m
                +2+1 // d
                +2+1 // h
                +2+1 // m
                +11+1// s
                +3   // flag
                +3;   // n_sat
            if line.len() > min_len { // RINEX3: clock offset precision was increased
                Some(line.split_at(min_len).1.trim()) // this handles it naturally
            } else {
                None
            }
        },
    };
    let clock_offset : Option<f64> = match offs.is_some() {
        true => {
            if let Ok(f) = f64::from_str(offs.unwrap()) {
                Some(f)
            } else {
                None // parsing failed for some reason
            }
        },
        false => None, // empty field
    };
	
	let data = match header.version.major {
		2 => {
			// grab system descriptions
			//  current line remainder
			//  and possible following lines
			// This remains empty on RINEX3, because we have such information
			// on following lines, which is much more convenient
			let mut systems = String::with_capacity(24*3); //SVNN
			let nb_sv_line: usize = num_integer::div_ceil(n_sat, 12).into();
			systems.push_str(rem.trim());
			for _ in 1..nb_sv_line {
				if let Some(l) = lines.next() {
					systems.push_str(l.trim());
				}
			}
			parse_v2(&systems, observables, lines)
		},
		_ => parse_v3(observables, lines),
	};
	Ok((epoch, clock_offset, data))
}

/* 
 * Parses a V2 epoch from given lines iteratoor
 * Vehicule description is contained in the epoch descriptor
 * Each vehicule content is wrapped into several lines
 */
fn parse_v2 (systems: &str, observables: &HashMap<Constellation, Vec<String>>, lines: std::str::Lines<'_>) -> BTreeMap<Sv, HashMap<String, ObservationData>> {
	let svnn_size = 3; // SVNN standard
	let nb_max_observables = 5; // in a single line
	let observable_width = 16; // data + 2 flags + 1 whitespace
	let nb_sat = systems.len() / svnn_size;
	let mut sv_ptr = 0; // svnn pointer
	let mut obs_ptr = 0; // observable pointer
	let mut sv = Sv::default(); // current vehicule we're dealing with 
	let mut obscodes : &Vec<String>;
	let mut data: BTreeMap<Sv, HashMap<String, ObservationData>> = BTreeMap::new();
	let mut inner: HashMap<String, ObservationData> = HashMap::with_capacity(5);
	if systems.len() < svnn_size {
		// Can't even parse a single vehicule;
		// epoch descriptor is totally corrupt, stop here
		return data ; 
	}
	// identify first vehicule
	if let Ok(ssv) = Sv::from_str(&systems[sv_ptr..sv_ptr+svnn_size]) {
		sv = ssv;
	}
	// observables for constellation context
	if let Some(observables) = observables.get(&sv.constellation) {
		obscodes = &observables;
	} else { // no observables for first vehicule found
		// while we could improve this and try to move on identifying next vehicule,
		// returning empty data simplify things
		return data ;
	}

	for line in lines { // browse all lines
		// parse line content
		let nb_obs = num_integer::div_ceil(line.len(), observable_width) ;
		//DEBUG
        //println!("LINE: \"{}\"", line);
		//println!("SV: \"{}\" PTR {}", sv, sv_ptr);
		//println!("NB OBS: {}", nb_obs);
		for i in 0..nb_obs {
			obs_ptr += 1 ;
			if obs_ptr <= obscodes.len() {
                let offset = i * observable_width;
                if line.len() > offset+observable_width-2 { // can parse an Obs
                    let observation_str = &line[offset..offset+observable_width-2];
                    if let Ok(obs) = f64::from_str(observation_str.trim()) {
                        let lli: Option<LliFlags>;
                        let ssi: Option<Ssi>;
                        if line.len() < offset + observable_width-1 {
                            // can't parse an LLI
                            // This only happens when omitted on last line
                            lli = None;
                            ssi = None;
                        } else { // enough content to parse an LLI
                            let lli_str = &line[offset+observable_width-2..offset+observable_width-1];
                            if let Ok(u) = u8::from_str_radix(lli_str.trim(), 10) {
                                lli = LliFlags::from_bits(u)
                            } else {
                                lli = None;
                            }

                            if line.len() < offset + observable_width {
                                // can't parse an SSI
                                // this only happens when omitted on last line
                                ssi = None;
                            } else {
                                let ssi_str = &line[offset+observable_width-1..offset+observable_width];
                                if let Ok(s) = Ssi::from_str(ssi_str.trim()) {
                                    ssi = Some(s)
                                } else {
                                    ssi = None;
                                }
                            }
                        }
                        //DEBUG
                        //println!("OBS {} LLI {:?} SSI {:?}", obs, lli, ssi);
                        inner.insert(
                            obscodes[obs_ptr-1].clone(), // key
                            ObservationData {
                                obs,
                                lli,
                                ssi,
                            }); // build content
                    } // f64::parsing OK
                } // enough content to parse an Obs 
            } // obs_ptr < obscodes.len(): line unexpectedly long
		}
		// manage possibly omitted data
		if nb_obs < nb_max_observables {
            // data was omitted
			if nb_obs == 0 {
				// line is completely empty
				let missing = std::cmp::min(obscodes.len() - obs_ptr, nb_max_observables);
				obs_ptr += missing ;
			} else {
				// got less than MAX obs
				if obs_ptr < obscodes.len() {
					// while we were expecting some
					obs_ptr += nb_max_observables ;
				}
			}
		}
		if obs_ptr >= obscodes.len() { // ">" is here to manage possible overflow on corrupted lines
			// we're done parsing observables
			if inner.len() > 0 {
				// build content
				data.insert(sv, inner.clone());
				inner.clear();
			}
            if sv_ptr < (nb_sat-1) * svnn_size {
				sv_ptr += svnn_size;
				// identify next vehicule
				if let Ok(ssv) = Sv::from_str(&systems[sv_ptr..sv_ptr+svnn_size]) {
					sv = ssv;
				} else {
                    break ; // identification failed
                }
				if let Some(observables) = observables.get(&sv.constellation) {
					obscodes = &observables;
				} else {
                    break ; // identification failed
                }
				obs_ptr = 0; // reset
			} else {
				break ; // sv_ptr > system description overflow 
			}
		}	
	} // for all lines provided
	data 
}

/* 
 * Parses a V3 epoch from given lines iteratoor
 * Format is much simpler, one vehicule is described in a single line
 */
fn parse_v3 (observables: &HashMap<Constellation, Vec<String>>, lines: std::str::Lines<'_>) -> BTreeMap<Sv, HashMap<String, ObservationData>> {
	let svnn_size = 3; // SVNN standard
	let observable_width = 15; // data + 2 flags
	let mut data: BTreeMap<Sv, HashMap<String, ObservationData>> = BTreeMap::new();
	let mut inner: HashMap<String, ObservationData> = HashMap::with_capacity(5);
	for line in lines { // browse all lines
        let (sv, rem) = line.split_at(svnn_size +1);
        if let Ok(sv) = Sv::from_str(sv) {
            if let Some(obscodes) = observables.get(&sv.constellation) {
                let nb_obs = num_integer::div_ceil(rem.len(), observable_width) ;
                inner.clear();
                //DEBUG
                //println!("LINE: \"{}\"", line);
                //println!("SV: \"{}\"", sv);
                //println!("NB OBS: {}", nb_obs);
                for i in 0..nb_obs {
                    if i < obscodes.len() {
                        let offset = i * observable_width;
                        if rem.len() > offset+observable_width-2 { // can parse an Obs
                            let observation_str = &rem[offset..offset+observable_width-2];
                            if let Ok(obs) = f64::from_str(observation_str.trim()) {
                                let lli: Option<LliFlags>;
                                let ssi: Option<Ssi>;
                                if line.len() < offset + observable_width-1 {
                                    // can't parse an LLI
                                    // This only happens when omitted on last line
                                    lli = None;
                                    ssi = None;
                                } else { // enough content to parse an LLI
                                    let lli_str = &rem[offset+observable_width-2..offset+observable_width-1];
                                    if let Ok(u) = u8::from_str_radix(lli_str.trim(), 10) {
                                        lli = LliFlags::from_bits(u)
                                    } else {
                                        lli = None;
                                    }

                                    if line.len() < offset + observable_width {
                                        // can't parse an SSI
                                        // this only happens when omitted on last line
                                        ssi = None;
                                    } else {
                                        let ssi_str = &rem[offset+observable_width-1..offset+observable_width];
                                        if let Ok(s) = Ssi::from_str(ssi_str.trim()) {
                                            ssi = Some(s)
                                        } else {
                                            ssi = None;
                                        }
                                    }
                                }
                                //DEBUG
                                //println!("OBS {} LLI {:?} SSI {:?}", obs, lli, ssi);
                                inner.insert(
                                    obscodes[i].clone(), // key
                                    ObservationData { // payload
                                        obs,
                                        lli,
                                        ssi,
                                    }); // build content
                            } // f64::parsing OK
                        } // enough content to parse an Obs 
                    } //else: line abnormally long,
                    //got more data than we expect, avoid overflowing
                }
                if inner.len() > 0 {
                    // build content we correctly identified
                    data.insert(sv, inner.clone());
                }
            }//got some observables to work with
        } // Sv::from_str failed()
    }//browse all lines
	data 
}

/// Writes epoch into given streamer 
pub fn write_epoch (
        epoch: &Epoch,
        clock_offset: &Option<f64>,
        data: &BTreeMap<Sv, HashMap<String, ObservationData>>,
        header: &Header,
        writer: &mut BufferedWriter,
    ) -> std::io::Result<()> {
    if header.version.major < 3 {
        write_epoch_v2(epoch, clock_offset, data, header, writer)
    } else {
        write_epoch_v3(epoch, clock_offset, data, header, writer)
    }
}

fn write_epoch_v3(
        epoch: &Epoch,
        clock_offset: &Option<f64>,
        data: &BTreeMap<Sv, HashMap<String, ObservationData>>,
        header: &Header,
        writer: &mut BufferedWriter,
    ) -> std::io::Result<()> {
    let mut lines = String::new();
    let obscodes = &header.obs
        .as_ref()
        .unwrap()
        .codes;
    lines.push_str("> ");
    lines.push_str(&epoch.to_string_obs_v3());
    lines.push_str(&format!("{:3}", data.len()));
    if let Some(data) = clock_offset {
        lines.push_str(&format!("{:12.4}", data)); 
    }
    lines.push_str("\n");
    
    for (sv, data) in data.iter() {
        lines.push_str(&format!("{} ", sv.to_string()));
        if let Some(obscodes) = obscodes.get(&sv.constellation) {
            for code in obscodes {
                if let Some(observation) = data.get(code) {
                    lines.push_str(&format!(" {:10.3}", observation.obs));
                    if let Some(flag) = observation.lli {
                        lines.push_str(&format!("{}", flag.bits()));
                    } else {
                        lines.push_str(" ");
                    }
                    if let Some(flag) = observation.ssi {
                        lines.push_str(&format!("{}", flag));
                    } else {
                        lines.push_str(" ");
                    }
                } else {
                    lines.push_str(&format!("               "));
                }
            }
        }
        lines.push_str("\n");
    }
    write!(writer, "{}", lines)
}

fn write_epoch_v2(
        epoch: &Epoch,
        clock_offset: &Option<f64>,
        data: &BTreeMap<Sv, HashMap<String, ObservationData>>,
        header: &Header,
        writer: &mut BufferedWriter,
    ) -> std::io::Result<()> {
    let obscodes = &header.obs
        .as_ref()
        .unwrap()
        .codes;
    write!(writer, " {} {:2}", epoch.to_string_obs_v2(), data.len())?;
    let mut index = 0;
    for (sv, _) in data {
        index += 1;
        if (index %13) == 0 {
            if let Some(data) = clock_offset {
                write!(writer, " {:9.1}", data)?;
            } else {
				write!(writer, "\n                                ")?;
			}
        }
		write!(writer, "{}", sv)?;
    }
	write!(writer, "\n ")?;
	let obs_per_line = 5;
    for (sv, observations) in data.iter() {
		if let Some(obscodes) = obscodes.get(&sv.constellation) {
			let mut index = 0;
			for obscode in obscodes {
				index += 1;
				if index == obs_per_line {
					index = 0;
					write!(writer, "\n ")?;
				}
				if let Some(observation) = observations.get(obscode) {
					// --> data is provided
                   	write!(writer, " {:10.3}", observation.obs)?;
					if let Some(flag) = observation.lli {
						// --> lli provided
                        write!(writer, "{}", flag.bits())?;
					} else { // LLI omitted
                        write!(writer, " ")?;
					}
                    if let Some(ssi) = observation.ssi {
                        // --> ssi provided
                        write!(writer, "{}", ssi)?;
                    } else { // SSI omitted
                        write!(writer, " ")?;
                    }
				} else {
					// --> data is not provided: BLANK
                    write!(writer, "               ")?;
				}
			}
		} // else: no observables declared in header
		// for constellation to which this vehicule is tied to: 
		// will never happen on valid RINEX
    }
	Ok(())
}
/*
    for (epoch, (clock_offset, sv)) in record.iter() {
        let date = epoch.date;
        let flag = epoch.flag;
        let vehicules = sv.keys();
        let nb_sv = vehicules.len(); 
        let obscodes = &header.obs.as_ref().unwrap().codes;
        // first line(s)
        //   Epoch + flag + svnn + possible clock offset
        match header.version.major {
            1|2 => {
                write!(writer, " {} ",  date.format("%y %m %d %H %M").to_string())?;
                write!(writer, " {}         ", date.time().second())?;
                write!(writer, " {}", flag)?; 
                write!(writer, " {}", nb_sv)?; 
                let nb_extra = nb_sv / 12;
                let mut index = 0;
                for vehicule in vehicules.into_iter() {
                    write!(writer, "{}", vehicule)?; 
                    if (index+1) % 12 == 0 {
                        if let Some(clock_offset) = clock_offset {
                            write!(writer, "{:3.9}", clock_offset)?
                        }
                        write!(writer, "\n                                ")?
                    }
                    index += 1
                }
                if nb_extra == 0 {
                    if let Some(clock_offset) = clock_offset {
                        let _ = write!(writer, "{:3.9}\n", clock_offset);
                    } else {
                        let _ = write!(writer, "\n");
                    }
                }
            },
            _ => { // Modern revisions 
                write!(writer, "> {} ",  date.format("%Y %m %d %H %M").to_string())?;
                write!(writer, " {}         ", date.time().second())?;
                write!(writer, " {} ", flag)?; 
                write!(writer, " {}", nb_sv)?; 
                if let Some(clock_offset) = clock_offset {
                    write!(writer, "{:.12}", clock_offset)?
                }
                write!(writer, "\n")?
            }
        }
        // epoch body
        let mut index = 0;
        for (sv, obs) in sv.iter() {
            let mut modulo = 5;
            if header.version.major > 2 {
                // modern RINEX
                modulo = 100000; // 'infinite': no wrapping
                    // we behave like CRX2RNX which does not respect the standards,
                    // we should wrap @ 80 once again
                let _ = write!(writer, "{} ", sv);
            } else {
                let _ = write!(writer, " ");
            }
            // observables for this constellation
            // --> respect header order and data might be missing
            let codes = &obscodes[&sv.constellation];
            for code in codes.iter() {
                if let Some(data) = obs.get(code) {
                    let _ = write!(writer, "{:13.3}", data.obs);
                    if let Some(lli) = data.lli {
                        let _ = write!(writer, "{}", lli.bits());
                    } else {
                        let _ = write!(writer, " ");
                    }
                    if let Some(ssi) = data.ssi {
                        let _ = write!(writer, "{}", ssi as u8);
                    } else {
                        let _ = write!(writer, " ");
                    }
                    if (index+1) % modulo == 0 {
                        let _ = write!(writer, "\n");
                    }
                    let _ = write!(writer, " ");
                } else {
                    // obs is missing, simply fill with whitespace
                    let _ = write!(writer, "                ");
                }
                index += 1
            }
            write!(writer, "\n")?
        }
    }
*/

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn ssi() {
        let ssi = Ssi::from_str("0").unwrap(); 
        assert_eq!(ssi, Ssi::DbHz0);
        assert_eq!(ssi.is_bad(), true);
        let ssi = Ssi::from_str("9").unwrap(); 
        assert_eq!(ssi.is_excellent(), true);
        let ssi = Ssi::from_str("10"); 
        assert_eq!(ssi.is_err(), true);
    }
    #[test]
    fn new_epoch() {
        assert_eq!(        
            is_new_epoch("95 01 01 00 00 00.0000000  0  7 06 17 21 22 23 28 31",
                Version {
                    major: 2,
                    minor: 0,
                }
            ),
            true
        );
        assert_eq!(        
            is_new_epoch("21700656.31447  16909599.97044          .00041  24479973.67844  24479975.23247",
                Version {
                    major: 2,
                    minor: 0,
                }
            ),
            false
        );
        assert_eq!(        
            is_new_epoch("95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
                Version {
                    major: 2,
                    minor: 0,
                }
            ),
            true
        );
        assert_eq!(        
            is_new_epoch("95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
                Version {
                    major: 3,
                    minor: 0,
                }
            ),
            false 
        );
        assert_eq!(        
            is_new_epoch("> 2022 01 09 00 00 30.0000000  0 40",
                Version {
                    major: 3,
                    minor: 0,
                }
            ),
            true 
        );
        assert_eq!(        
            is_new_epoch("> 2022 01 09 00 00 30.0000000  0 40",
                Version {
                    major: 2,
                    minor: 0,
                }
            ),
            false
        );
        assert_eq!(        
            is_new_epoch("G01  22331467.880   117352685.28208        48.950    22331469.28",
                Version {
                    major: 3,
                    minor: 0,
                }
            ),
            false
        );
    }
}
