use thiserror::Error;
use std::str::FromStr;
//use chrono::Timelike;
use bitflags::bitflags;
use std::collections::{BTreeMap, HashMap};

use crate::{
    sv, Sv,
    constellation, Constellation,
    epoch,
    Epoch, EpochFlag,
    Header,
    version::Version,
    merge, merge::Merge,
    split, split::Split,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse epoch")]
    EpochError(#[from] epoch::Error),
    #[error("failed to parse epoch flag")]
    FlagError(#[from] epoch::flag::Error),
    #[error("failed to identify constellation")]
    ConstellationError(#[from] constellation::Error),
    #[error("failed to parse sv")]
    SvError(#[from] sv::Error),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse vehicules properly (nb_sat mismatch)")]
    EpochParsingError,
    #[error("line is empty")]
    MissingData,
}

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

/// Returns true if given content matches a new OBSERVATION data epoch
pub fn is_new_epoch (line: &str, v: Version) -> bool {
    let parsed: Vec<&str> = line
        .split_ascii_whitespace()
        .collect();
	if v.major < 3 {
        if line.len() < 30 {
            false
        } else {
            Epoch::from_str(&line[0..29]).is_ok()
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

    let epoch = Epoch::from_str(date)?;
    let flag = EpochFlag::from_str(flag.trim())?;
    let epoch = epoch.with_flag(flag);
	
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
			systems.push_str(rem.trim());
            let mut nb_extra_lines: usize = 0;
            if n_sat > 12 && n_sat <= 24 {
                nb_extra_lines = 1;
            } else if n_sat > 24 && n_sat <= 36 {
                nb_extra_lines = 2;
            } else if n_sat > 36 && n_sat <= 48 {
                nb_extra_lines = 3;
            } else if n_sat > 48 && n_sat <= 60 {
                nb_extra_lines = 4;
            } else if n_sat > 60 && n_sat <= 78 {
                nb_extra_lines = 5;
            }
            for _ in 0..nb_extra_lines {
                if let Some(l) = lines.next() {
                    systems.push_str(l.trim());
                } else {
                    return Err(Error::MissingData) ;
                }
			}
			parse_v2(&header, &systems, observables, lines)
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
fn parse_v2 (header: &Header, systems: &str, observables: &HashMap<Constellation, Vec<String>>, lines: std::str::Lines<'_>) -> BTreeMap<Sv, HashMap<String, ObservationData>> {
	let svnn_size = 3; // SVNN standard
	let nb_max_observables = 5; // in a single line
	let observable_width = 16; // data + 2 flags + 1 whitespace
	let mut sv_ptr = 0; // svnn pointer
	let mut obs_ptr = 0; // observable pointer
	let mut data: BTreeMap<Sv, HashMap<String, ObservationData>> = BTreeMap::new();
	let mut inner: HashMap<String, ObservationData> = HashMap::with_capacity(5);
    let mut sv: Sv;
    let mut obscodes : &Vec<String>;
    //println!("SYSTEMS \"{}\"", systems); // DEBUG

    // parse first system we're dealing with
    if systems.len() < svnn_size {
		// Can't even parse a single vehicule;
		// epoch descriptor is totally corrupt, stop here
		return data ; 
	}

    let max = std::cmp::min(svnn_size, systems.len()); // covers epoch with a unique vehicule
    let system = &systems[0..max];
    //println!("SYSTEM \"{}\"", system); //DEBUG
    // parse 1st system to work on
    if let Ok(ssv) = Sv::from_str(system) {
        sv = ssv;
    } else {
        // mono constellation context
        if let Ok(prn) = u8::from_str_radix(system.trim(), 10) {
            if let Some(constellation) = header.constellation {
                sv = Sv {
                    constellation,
                    prn,
                }
            } else {
                panic!("faulty RINEX2 constellation /sv definition");
            }
        } else {
            // can't parse 1st vehicule
            return data ;
        }
    }
    sv_ptr += svnn_size; // increment pointer
    // grab observables for this vehicule
    if let Some(observables) = observables.get(&sv.constellation) {
        obscodes = &observables;
    } else {
        // failed to identify observations for this vehicule
        return data ;
    }

	for line in lines { // browse all lines provided
        let line_width = line.len();
        if line_width < 10 {
            //println!("\nEMPTY LINE: \"{}\"", line); //DEBUG
            // line is empty 
            // add maximal amount of vehicules possible
            obs_ptr += std::cmp::min(nb_max_observables, obscodes.len() - obs_ptr);
            // nothing to parse
        } else {
            // not a empty line
            //println!("\nLINE: \"{}\"", line); //DEBUG
            let nb_obs = num_integer::div_ceil(line_width, observable_width) ; // nb observations in this line
            //println!("NB OBS: {}", nb_obs); //DEBUG
            // parse all obs
            for i in 0..nb_obs {
                obs_ptr += 1;
                if obs_ptr > obscodes.len() {
                    // line is abnormally long compared to header definitions
                    //  parsing would fail
                    break ;
                }
                let slice: &str = match i {
                    0 => {
                        &line[0..std::cmp::min(17, line_width)] // manage trimmed single obs
                    },
                    _ => {
                        let start = i*observable_width;
                        let end = std::cmp::min((i+1)*observable_width, line_width); // trimmed lines
                        &line[start..end]
                    },
                };
                //println!("WORK CONTENT \"{}\"", slice); //DEBUG
                //TODO: improve please
                let obs = &slice[0..std::cmp::min(slice.len(), 14)]; // trimmed observations
                //println!("OBS \"{}\"", obs); //DEBUG
                let mut lli: Option<LliFlags> = None;
                let mut ssi: Option<Ssi> = None;
                if let Ok(obs) = f64::from_str(obs.trim()) { // parse obs
                    if slice.len() > 14 {
                        let lli_str = &slice[14..15];
                        if let Ok(u) = u8::from_str_radix(lli_str, 10) {
                            lli  = LliFlags::from_bits(u);
                        }
                        if slice.len() > 15 { 
                            let ssi_str = &slice[15..16];
                            if let Ok(s) = Ssi::from_str(ssi_str) {
                                ssi = Some(s);
                            }
                        }
                    }
                    //println!("{} {:?} {:?} ==> {}", obs, lli, ssi, obscodes[obs_ptr-1]); //DEBUG
                    inner.insert(
                        obscodes[obs_ptr-1].clone(),
                        ObservationData {
                            obs,
                            lli,
                            ssi,
                        });
                }//f64::obs
            } // parsing all observations 
        }

        if obs_ptr >= obscodes.len() {
            // we're done with current vehicule
            // build data
            data.insert(sv, inner.clone());
            inner.clear(); // prepare for next vehicule
            obs_ptr = 0;
            //identify next vehicule
            if sv_ptr >= systems.len() {
                // that was the last vehicule 
                return data;
            }
            // identify next vehicule 
            let start = sv_ptr;
            let end = std::cmp::min(sv_ptr + svnn_size, systems.len()); // trimed epoch description
            let system = &systems[start..end];
            //println!("NEW SYSTEM \"{}\"\n", system); //DEBUG
            if let Ok(ssv) = Sv::from_str(system) {
                sv = ssv;
            } else {
                // mono constellation context
                if let Ok(prn) = u8::from_str_radix(system.trim(), 10) {
                    if let Some(constellation) = header.constellation {
                        sv = Sv {
                            constellation,
                            prn,
                        }
                    } else {
                        panic!("faulty RINEX2 constellation /sv definition");
                    }
                } else {
                    // can't parse vehicule
                    return data ;
                }
            }
            sv_ptr += svnn_size; // increment pointer
            // grab observables for this vehicule
            if let Some(observables) = observables.get(&sv.constellation) {
                obscodes = &observables;
            } else {
                // failed to identify observations for this vehicule
                return data ;
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
	let observable_width = 16; // data + 2 flags
	let mut data: BTreeMap<Sv, HashMap<String, ObservationData>> = BTreeMap::new();
	let mut inner: HashMap<String, ObservationData> = HashMap::with_capacity(5);
	for line in lines { // browse all lines
        let (sv, line) = line.split_at(svnn_size);
        if let Ok(sv) = Sv::from_str(sv) {
            if let Some(obscodes) = observables.get(&sv.constellation) {
                let nb_obs = num_integer::div_ceil(line.len(), observable_width); 
                inner.clear();
                //println!("LINE: \"{}\"", line); //DEBUG
                //println!("SV: \"{}\"", sv); //DEBUG
                //println!("NB OBS: {}", nb_obs); //DEBUG
                let mut rem = line;
                for i in 0..nb_obs {
                    if i > obscodes.len() {
                        break ; // line abnormally long
                            // does not match previous Header definitions
                            // => would not be able to sort data
                    }
                    // avoids overflowing
                    let split_offset = std::cmp::min(observable_width, rem.len());
                    let (content, r) = rem.split_at(split_offset);
                    //println!("content \"{}\" \"{}\"", content, r);
                    rem = r.clone();
                    let content_len = content.len();
                    let mut ssi: Option<Ssi> = None;
                    let mut lli: Option<LliFlags> = None;
                    let obs = &content[0..std::cmp::min(observable_width-2, content_len)];
                    //println!("OBS \"{}\"", obs); //DEBUG
                    if let Ok(obs) = f64::from_str(obs.trim()) {
                        if content_len > observable_width-2 {
                            let lli_str = &content[observable_width-2..observable_width-1];
                            if let Ok(u) = u8::from_str_radix(lli_str, 10) {
                                lli = LliFlags::from_bits(u);
                            }
                        }
                        if content_len > observable_width-1 {
                            let ssi_str = &content[observable_width-1..observable_width];
                            if let Ok(s) = Ssi::from_str(ssi_str) {
                                ssi = Some(s);
                            }
                        }
                        //println!("LLI {:?}", lli); //DEBUG
                        //println!("SSI {:?}", ssi);
                        // build content
                        inner.insert(
                            obscodes[i].clone(),
                            ObservationData {
                                obs,
                                lli,
                                ssi,
                            },
                        );
                    }
                }
                if inner.len() > 0 {
                    data.insert(sv, inner.clone());
                }
            }//got some observables to work with
        } // Sv::from_str failed()
    }//browse all lines
	data 
}

/// Formats one epoch according to standard definitions 
pub fn fmt_epoch (
        epoch: &Epoch,
        clock_offset: &Option<f64>,
        data: &BTreeMap<Sv, HashMap<String, ObservationData>>,
        header: &Header,
    ) -> String {
    if header.version.major < 3 {
        fmt_epoch_v2(epoch, clock_offset, data, header)
    } else {
        fmt_epoch_v3(epoch, clock_offset, data, header)
    }
}

fn fmt_epoch_v3(
        epoch: &Epoch,
        clock_offset: &Option<f64>,
        data: &BTreeMap<Sv, HashMap<String, ObservationData>>,
        header: &Header,
    ) -> String { 
    let mut lines = String::with_capacity(128);
    let obscodes = &header.obs
        .as_ref()
        .unwrap()
        .codes;
    lines.push_str(&format!("> {} {:2}", epoch, data.len()));
    if let Some(data) = clock_offset {
        lines.push_str(&format!("{:13.4}", data)); 
    }
    lines.push_str("\n");
    
    for (sv, data) in data.iter() {
        lines.push_str(&format!("{}", sv.to_string()));
        if let Some(obscodes) = obscodes.get(&sv.constellation) {
            for code in obscodes {
                if let Some(observation) = data.get(code) {
                    lines.push_str(&format!("{:14.3}", observation.obs));
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
                    lines.push_str(&format!("                "));
                }
            }
        }
        lines.push_str("\n");
    }
    lines
}

fn fmt_epoch_v2(
        epoch: &Epoch,
        clock_offset: &Option<f64>,
        data: &BTreeMap<Sv, HashMap<String, ObservationData>>,
        header: &Header,
    ) -> String { 
    let mut lines = String::with_capacity(128);
    let obscodes = &header.obs
        .as_ref()
        .unwrap()
        .codes;
    lines.push_str(&format!(" {} {:2}", epoch, data.len()));
    let mut index = 0;
    for (sv, _) in data {
        index += 1;
        if (index %13) == 0 {
            if let Some(data) = clock_offset {
                lines.push_str(&format!(" {:9.1}", data));
            } else {
				lines.push_str(&format!("\n                                "));
			}
        }
		lines.push_str(&sv.to_string());
    }
	let obs_per_line = 5;
    // for each vehicule per epoch
    for (sv, observations) in data.iter() {
        // follow list of observables, as described in header section
        // for given constellation 
		if let Some(obscodes) = obscodes.get(&sv.constellation) {
			for (obs_index, obscode) in obscodes.iter().enumerate() {
				if obs_index % obs_per_line == 0 {
                    lines.push_str("\n");
                }
				if let Some(observation) = observations.get(obscode) {
                    let formatted_obs = format!("{:14.3}", observation.obs);
                    let formatted_flags: String = match observation.lli {
                        Some(lli) => {
                            match observation.ssi {
                                Some(ssi) => format!("{}{}", lli.bits(), ssi),
                                _ => format!("{} ", lli.bits()),
                            }
                        },
                        _ => {
                            match observation.ssi {
                                Some(ssi) => format!(" {}", ssi),
                                _ => "  ".to_string(),
                            }
                        },
                    };
                    lines.push_str(&formatted_obs);
                    lines.push_str(&formatted_flags);
				} else {
					// --> data is not provided: BLANK
                    lines.push_str("                ");
				}
			}
		} 
    }
    lines.push_str("\n");
    lines
}

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

impl Merge<Record> for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        for (epoch, (clk_offset, vehicules)) in rhs.iter() {
            if let Some((cclk_offset, vvehicules)) = self.get_mut(epoch) {
                if let Some(clk_offset) = clk_offset {
                    if cclk_offset.is_none() {
                        *cclk_offset = Some(*clk_offset); // clock offset is now provided
                    }
                }
                for (vehicule, observations) in vehicules.iter() {
                    if let Some(oobservations) = vvehicules.get_mut(vehicule) {
                        for (observable, data) in observations.iter() {
                            if let Some(ddata) = oobservations.get_mut(observable) {
                                // observation both provided
                                //  fill missing flags but leave data untouched
                                if let Some(lli) = data.lli {
                                    if ddata.lli.is_none() {
                                        ddata.lli = Some(lli);
                                    }
                                }
                                if let Some(ssi) = data.ssi {
                                    if ddata.ssi.is_none() {
                                        ddata.ssi = Some(ssi);
                                    }
                                }
                            } else { //new observation
                                oobservations.insert(observable.clone(), data.clone());
                            }
                        }
                    } else { // new vehicule
                        vvehicules.insert(*vehicule, observations.clone());
                    }
                }
            } else { // new epoch
                self.insert(*epoch, (*clk_offset, vehicules.clone()));
            }
        }
        Ok(())
    }
}

impl Split<Record> for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self.iter()
            .flat_map(|(k, v)| {
                if k < &epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self.iter()
            .flat_map(|(k, v)| {
                if k >= &epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((r0, r1))
    }
}
