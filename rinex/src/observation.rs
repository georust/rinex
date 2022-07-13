//! `ObservationData` parser and related methods
use std::io::Write;
use thiserror::Error;
use crate::version;
use std::str::FromStr;
use std::collections::{BTreeMap, HashMap};
use physical_constants::SPEED_OF_LIGHT_IN_VACUUM;

use crate::sv;
use crate::epoch;
use crate::header;
use crate::constellation;
use crate::constellation::Constellation;

/// Describes `Compact RINEX` specific information
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct Crinex {
    /// Compression program version
    pub version: version::Version,
    /// Compression program name
    pub prog: String,
    /// Date of compression
    #[cfg_attr(feature = "with-serde", serde(with = "datetime_formatter"))]
    pub date: chrono::NaiveDateTime,
}

/// Describes known marker types
/// Observation Record specific header fields
#[derive(Debug, Clone)]
pub struct HeaderFields {
    /// Optional CRINEX information,
    /// only present on compressed OBS
    pub crinex: Option<Crinex>, 
    /// Observation codes present in this file, by Constellation
    pub codes: HashMap<Constellation, Vec<String>>,
    /// True if epochs & data compensate for local clock drift
    pub clock_offset_applied: bool,
}

/// `Ssi` describes signals strength
#[repr(u8)]
#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
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
    DbHz21_29 = 4, 
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

impl Default for Ssi {
    fn default() -> Ssi { Ssi::DbHz54 }
}

impl std::str::FromStr for Ssi {
    type Err = std::io::Error;
    fn from_str (code: &str) -> Result<Self, Self::Err> {
        match code {
            "0" => Ok(Ssi::DbHz0),
            "1" => Ok(Ssi::DbHz12),
            "2" => Ok(Ssi::DbHz12_17),
            "3" => Ok(Ssi::DbHz18_23),
            "4" => Ok(Ssi::DbHz21_29),
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

pub mod lli_flags {
    /// Current epoch is marked Ok or Unknown status 
    pub const OK_OR_UNKNOWN : u8 = 0x00;
    /// Lock lost between previous observation and current observation,
    /// cycle slip is possible
    pub const LOCK_LOSS : u8 = 0x01;
    /// Opposite wavelenght factor to the one defined
    /// for the satellite by a previous WAVELENGTH FACT comment,
    /// or opposite to default value, is not previous WAVELENFTH FACT comment
    pub const HALF_CYCLE_SLIP : u8 = 0x02;
    /// Observing under anti spoofing,
    /// might suffer from decreased SNR - decreased signal quality
    pub const UNDER_ANTI_SPOOFING : u8 = 0x04;
}

#[derive(PartialEq, Copy, Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ObservationData {
	/// physical measurement
	pub obs: f32,
	/// Lock loss indicator 
	pub lli: Option<u8>,
	/// Signal strength indicator
	pub ssi: Option<Ssi>,
}

impl ObservationData {
	/// Builds new ObservationData structure from given predicates
    pub fn new (obs: f32, lli: Option<u8>, ssi: Option<Ssi>) -> ObservationData {
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
    ///    + LLI must match the lli_flags::OkOrUnknown flag (strictly)    
    /// if SSI exists:    
    ///    + SSI must match the .is_ok() criteria, refer to API 
    pub fn is_ok (self) -> bool {
        let lli_ok = self.lli.unwrap_or(lli_flags::OK_OR_UNKNOWN) == lli_flags::OK_OR_UNKNOWN;
        let ssi_ok = self.ssi.unwrap_or(Ssi::default()).is_ok();
        lli_ok && ssi_ok
    }
}

/// `Record` content for OBS data files.   
/// Measurements are sorted by `epoch` (timestamps + flags).    
/// Measurements are of two kinds:
///  + Option<f32>: receiver clock offsets for OBS data files where   
///    receiver clock offsets are 'applied'    
///  + map of ObservationData (physical measurements) sorted by `Sv` and by observation codes 
pub type Record = BTreeMap<epoch::Epoch, 
    (Option<f32>, 
    HashMap<sv::Sv, HashMap<String, ObservationData>>)>;

#[derive(Error, Debug)]
/// OBS Data `Record` parsing specific errors
pub enum RecordError {
    #[error("failed to parse date")]
    ParseDateError(#[from] epoch::ParseDateError),
    #[error("failed to parse epoch flag")]
    ParseEpochFlagError(#[from] std::io::Error),
    #[error("failed to parse sv")]
    SvError(#[from] sv::Error),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse vehicules properly (n_sat mismatch)")]
    EpochParsingError,
}

/// Builds `Record` entry for `ObservationData`
/// from given epoch content
pub fn build_record_entry (header: &header::Header, content: &str)
        -> Result<(epoch::Epoch, Option<f32>, HashMap<sv::Sv, HashMap<String, ObservationData>>), RecordError> 
{
    let mut lines = content.lines();
    let mut line = lines.next()
        .unwrap();

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
    let (n_sat, mut rem) = rem.split_at(3);
    let n_sat = u16::from_str_radix(n_sat.trim(), 10)?;
    let n_sv_line : usize = num_integer::div_ceil(n_sat, 12).into();

    let flag = epoch::EpochFlag::from_str(flag.trim())?;
    let date = epoch::str2date(date)?; 
    let epoch = epoch::Epoch::new(date, flag);

    let mut sv_list : Vec<sv::Sv> = Vec::with_capacity(24);
	let mut map : HashMap<sv::Sv, HashMap<String, ObservationData>> = HashMap::new();
	
    // all encountered obs codes
    let obs = header.obs
        .as_ref()
        .unwrap();
    let obs_codes = &obs.codes;
    
    // grabbing possible clock_offsets content
    let offs : Option<&str> = match header.version.major < 2 {
        true => {
            // old fashion RINEX:
            // clock offsets are last 12 characters
            if line.len() > 60-12 {
                Some(line.split_at(60-12).1.trim())
            } else {
                None
            }
        },
        false => {
            // modern RINEX:
            let min_len : usize = 
                 4+1 // y
                +2+1 // m
                +2+1 // d
                +2+1 // h
                +2+1 // m
                +11+1// s
                +3   // flag
                +3;   // n_sat
            if line.len() > min_len {
                Some(line.split_at(min_len).1.trim()) // increased precision
            } else {
                None
            }
        },
    };
    let clock_offset : Option<f32> = match offs.is_some() {
        true => {
            if let Ok(f) = f32::from_str(offs.unwrap()) {
                Some(f)
            } else {
                None // parsing failed for some reason
            }
        },
        false => None, // empty field
    };

    if header.version.major < 3 {
        // old fashion:
        //   Sv list is passed on 1st and possible several lines
        let mut offset : usize = 0;
        for _ in 0..n_sv_line {
            loop {
                let sv_str = &rem[offset..offset+3];
                let identifier = sv_str.chars().nth(0)
                    .unwrap(); 
                let prn = u8::from_str(&sv_str[1..].trim())?;
                // build `sv` 
                let sv : sv::Sv = match identifier.is_ascii_whitespace() {
                    true => sv::Sv::new(header.constellation.unwrap(), prn),
                    false => {
                        let constell : constellation::Constellation = match identifier {
                            'G' => constellation::Constellation::GPS,
                            'R' => constellation::Constellation::Glonass,
                            'J' => constellation::Constellation::QZSS,
                            'E' => constellation::Constellation::Galileo,
                            'C' => constellation::Constellation::Beidou,
                            'S' => constellation::Constellation::Sbas(constellation::Augmentation::default()),
                            _ => return Err(
                                RecordError::SvError(
                                    sv::Error::ConstellationError(
                                        constellation::Error::UnknownCode(identifier.to_string())))),
                        };
                        sv::Sv::new(constell, prn)
                    },
                };
                
                sv_list.push(sv);
                offset += 3;
                if offset == rem.len() {
                    line = lines.next()
                        .unwrap();
                    rem = line.trim();
                    offset = 0;
                    break
                }
            } // sv systems content 
        } // sv system ID
    
        // verify identified list sanity
        if sv_list.len() != n_sat.into() {
            return Err(RecordError::EpochParsingError) // mismatch
        }

		for i in 0..sv_list.len() { // per vehicule
			let mut offset : usize = 0;
			let mut obs_map : HashMap<String, ObservationData> = HashMap::new();

			// old RINEX revision : using previously identified Sv 
			let sv : sv::Sv = sv_list[i]; 
			let codes =  obs_codes
                .get(&sv.constellation)
                .unwrap();
			let mut code_index : usize = 0;
			loop { // per obs code
				let code = &codes[code_index];
				let obs : Option<f32> = match line.len() < offset+14 { 
					true => {
						// cant' grab a new measurement
						//  * line is empty: contains only empty measurements
						//  * end of line is reached
						None
					},
					false => {
						let obs = &line[offset..offset+14];
						if let Ok(f) = f32::from_str(&obs.trim()) {
							Some(f)
						} else {
							None // empty field
						}
					},
				};

				let lli : Option<u8> = match line.len() < offset+14+1 {
					true => {
						// can't parse lli here
						// 	* line is over and this measurement
						//    does not have lli nor ssi 
						None
					},
					false => {
						let lli = &line[offset+14..offset+14+1];
						let lli = match u8::from_str_radix(&lli, 10) {
							Ok(lli) => Some(lli),
							Err(_) => None, // lli field is empty
						};
						lli
					},
				};

				let ssi : Option<Ssi> = match line.len() < offset+14+2 {
					true => {
						// can't parse ssi here
						// 	* line is over and this measurement
						//    does not have ssi 
						None
					},
					false => {
						let ssi = &line[offset+14+1..offset+14+2];
						let ssi = match Ssi::from_str(ssi) {
							Ok(ssi) => Some(ssi),
							Err(_) => None, // ssi field is empty
						};
						ssi
					},
				};
				
				if let Some(obs) = obs { // parsed something
					let obs = ObservationData::new(obs, lli, ssi);
					obs_map.insert(code.to_string(), obs); 
				}
				
				code_index += 1;
				if code_index == obs_codes.len() {
					break // last code that system sv
				}
				
				offset += 14 // F14.3
					+1  // +lli
					+1; // +ssi

				if offset >= line.len() {
					// we just parsed the last
					// code for this line
					offset = 0;
					if let Some(l) = lines.next() {
						line = l;
					}
				}
			} // for all obs code
            map.insert(sv, obs_map);
			if let Some(l) = lines.next() {
				line = l;
			} else {
				break
			}
		} // for all systems
    } // V < 3 old fashion
	else { // V > 2 modern RINEX
		for _ in 0..n_sat {
			if let Some(l) = lines.next() {
				line = l;
			} else {
				break
			}
			
			// parse Sv and identify
			let (sv, rem) = line.split_at(3);
			let identifier = sv.chars().nth(0)
				.unwrap();
			let prn = u8::from_str_radix(&sv[1..].trim(),10)?;
			let constell : constellation::Constellation = match identifier {
				'G' => constellation::Constellation::GPS,
				'R' => constellation::Constellation::Glonass,
				'J' => constellation::Constellation::QZSS,
				'E' => constellation::Constellation::Galileo,
				'C' => constellation::Constellation::Beidou,
				'H' => constellation::Constellation::Sbas(constellation::Augmentation::default()),
                'S' => constellation::Constellation::Geo,
				_ => return Err(
                        RecordError::SvError(
                            sv::Error::ConstellationError(
                                constellation::Error::UnknownCode(identifier.to_string())))),
			};
			let sv = sv::Sv::new(constell, prn);
			// retrieve obs code for that system
			let codes =  &obs_codes[&constell];
			let mut offset : usize = 0;
			let mut code_index : usize = 0;
			let mut obs_map : HashMap<String, ObservationData> = HashMap::new();
			loop { // per obs code
				let code = &codes[code_index];
				let obs = &rem[offset..offset+14];
				let obs : Option<f32> = match f32::from_str(&obs.trim()) {
					Ok(f) => Some(f),
					Err(_) => None, // empty field
				};
				let lli : Option<u8> = match rem.len() < offset+14+1 {
					true => {
						// can't parse lli here,
						// line is terminated by an OBS without lli nor ssi
						None
					},
					false => {
						let lli = &rem[offset+14..offset+14+1];
						let lli = match u8::from_str_radix(&lli, 10) {
							Ok(lli) => Some(lli),
							Err(_) => None, // lli field is empty
						};
						lli
					},
				};
				let ssi : Option<Ssi> = match rem.len() < offset+14+2 {
					true => {
						// can't parse ssi here,
						// line is terminated by an OBS without ssi
						None
					},
					false => {
						let ssi = &rem[offset+14+1..offset+14+2];
						let ssi = match Ssi::from_str(ssi) {
							Ok(ssi) => Some(ssi),
							Err(_) => None, // ssi field is empty
						};
						ssi
					},
				};

				if let Some(obs) = obs { // parsed something
					let obs = ObservationData::new(obs, lli, ssi);
					obs_map.insert(code.to_string(), obs);
					code_index += 1;
				}
				
				offset += 14 // F14.3
					+1  // +lli
					+1; // +ssi
				
				if offset >= rem.len() { // done parsing this line
					map.insert(sv, obs_map);
					break
				}
			} // per obs code
		} // per sat
	} // V>2
    Ok((epoch, clock_offset, map))
}

/// Pushes observation record into given file writer
pub fn to_file (header: &header::Header, record: &Record, mut writer: std::fs::File) -> std::io::Result<()> {
    for (epoch, (clock_offset, observations)) in record.iter() {
        match header.version.major {
            1|2 => {
                write!(writer, " {} ",  epoch.date.format("%y %m %d %H %M %.6f").to_string())?;
                //TODO wrapp systems on as many lines as needed
                if let Some(clock_offset) = clock_offset {
                    write!(writer, "{:.12}", clock_offset)?
                }
            },
            _ => {
                write!(writer, "> {} ", epoch.date.format("%Y %m %d %H %M %.6f").to_string())?;
                if let Some(clock_offset) = clock_offset {
                    write!(writer, "{:.12}", clock_offset)?
                }
                write!(writer, "\n")?
            }
        }
        for (sv, _obs) in observations.iter() {
            if header.version.major > 2 {
                // modern RINEX
                write!(writer, "{} ", sv)?
            }
            /*for code in &obs_codes[&sv.constellation] { 
                let data = obs[code];
                write!(writer, "{:14.3} ", data.obs)?;
                if let Some(lli) = data.lli {
                    write!(writer, "{} ", lli)?
                } else {
                    write!(writer, " ")?
                }
            }*/
            write!(writer, "\n")?
        }
    }
    Ok(())
}

/// Calculates distance from given Pseudo Range value,
/// by compensating clock offsets    
/// pseudo_rg: raw pseudo range measurements   
/// rcvr_clock_offset: receiver clock offset (s)    
/// sv_clock_offset: Sv clock offset (s)    
/// biases: optionnal (additive) biases to compensate for and increase result accuracy 
pub fn pseudo_range_to_distance (pseudo_rg: f64, rcvr_clock_offset: f64, sv_clock_offset: f64, _biases: Vec<f64>) -> f64 {
    pseudo_rg - SPEED_OF_LIGHT_IN_VACUUM * (rcvr_clock_offset - sv_clock_offset)
    //TODO handle biases
    // p17 table 4
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    /// Tests `Ssi` constructor
    fn test_ssi() {
        let ssi = Ssi::from_str("0").unwrap(); 
        assert_eq!(ssi, Ssi::DbHz0);
        assert_eq!(ssi.is_bad(), true);
        let ssi = Ssi::from_str("9").unwrap(); 
        assert_eq!(ssi.is_excellent(), true);
        let ssi = Ssi::from_str("10"); 
        assert_eq!(ssi.is_err(), true);
    }
    #[test]
    /// Tests `Lli` masking operation
    fn test_lli_masking() {
        let lli = 0;
        assert_eq!(lli & lli_flags::OK_OR_UNKNOWN, 0);
        let lli = 0x03;
        assert_eq!(lli & lli_flags::LOCK_LOSS > 0, true); 
        assert_eq!(lli & lli_flags::HALF_CYCLE_SLIP > 0, true); 
        let lli = 0x04;
        assert_eq!(lli & lli_flags::UNDER_ANTI_SPOOFING > 0, true); 
    }
}
