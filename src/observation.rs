//! `ObservationData` parser and related methods
use thiserror::Error;
use std::str::FromStr;
use strum_macros::EnumString;
use std::collections::HashMap;
use physical_constants::SPEED_OF_LIGHT_IN_VACUUM;
    
use crate::epoch;
use crate::record;
use crate::record::Sv;
use crate::constellation;
use crate::header::RinexHeader;

#[macro_export]
/// Returns True if 3 letter code 
/// matches a pseudo range (OBS) code
macro_rules! is_pseudo_range_obs_code {
    ($code: expr) => { 
        $code.starts_with("C") || $code.starts_with("P") // non gps old fashion
    };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a phase (OBS) code
macro_rules! is_phase_carrier_obs_code {
    ($code: expr) => { $code.starts_with("L") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a doppler (OBS) code
macro_rules! is_doppler_obs_code {
    ($code: expr) => { $code.starts_with("D") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a signal strength (OBS) code
macro_rules! is_sig_strength_obs_code {
    ($code: expr) => { $code.starts_with("S") };
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct ObservationData {
	/// physical measurement
	obs: f32,
	/// Lock loss indicator 
	lli: Option<u8>,
	/// Signal strength indicator
	ssi: Option<u8>,
}

impl ObservationData {
	pub fn new (obs: f32, lli: Option<u8>, ssi: Option<u8>) -> ObservationData {
		ObservationData {
			obs,
			lli,
			ssi,
		}
	}
}

/// `Record` content for OBS data files.   
/// Measurements are sorted by `epoch` (timestamps + flags).    
/// Measurements are of two kinds:
///  + Option<f32>: receiver clock offsets for OBS data files where   
///    receiver clock offsets are 'applied'    
///  + map of ObservationData (physical measurements) sorted by `Sv` 
pub type Record = HashMap<epoch::Epoch, 
    (Option<f32>, 
    HashMap<Sv, HashMap<String, ObservationData>>)>;

/// Calculates distance from given Pseudo Range value,
/// by compensating clock offsets    
/// pr: raw pseudo range measurements   
/// rcvr_clock_offset: receiver clock offset (s)    
/// sat_clock_offset: Sv clock offset (s)    
/// biases: other additive biases
pub fn distance_from_pseudo_range (pr: f64,
    rcvr_clock_offset: f64, sat_clock_offset: f64, biases: Vec<f64>)
        -> f64 {
    pr - SPEED_OF_LIGHT_IN_VACUUM * (rcvr_clock_offset - sat_clock_offset)
    // modulo leap second?
    // p17 table 4
}

#[derive(Error, Debug)]
/// OBS Data `Record` parsing specific errors
pub enum RecordError {
    #[error("failed to parse date")]
    ParseDateError(#[from] epoch::ParseDateError),
    #[error("failed to parse epoch flag")]
    ParseEpochFlagError(#[from] std::io::Error),
    #[error("failed to parse sv")]
    ParseSvError(#[from] record::ParseSvError),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse vehicules properly (n_sat mismatch)")]
    EpochParsingError,
}

/// Builds `RINEX` record entry for `Observation` Data files.    
/// Returns identified `epoch` to later sort data efficiently.    
/// Returns 2D data as described in `record` definition
pub fn build_record_entry (header: &RinexHeader, content: &str)
        -> Result<(epoch::Epoch, Option<f32>, HashMap<Sv, HashMap<String, ObservationData>>), RecordError> 
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

    let mut sv_list : Vec<Sv> = Vec::with_capacity(24);
	let mut map : HashMap<Sv, HashMap<String, ObservationData>> = HashMap::new();
    
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
                let sv : Sv = match identifier.is_ascii_whitespace() {
                    true => Sv::new(header.constellation.unwrap(), prn),
                    false => {
                        let constell : constellation::Constellation = match identifier {
                            'G' => constellation::Constellation::GPS,
                            'R' => constellation::Constellation::Glonass,
                            'J' => constellation::Constellation::QZSS,
                            'E' => constellation::Constellation::Galileo,
                            'C' => constellation::Constellation::Beidou,
                            'S' => constellation::Constellation::Sbas,
                            _ => return Err(
                                RecordError::ParseSvError(
                                    record::ParseSvError::UnidentifiedConstellation(identifier))),
                        };
                        Sv::new(constell, prn)
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
			let sv : Sv = sv_list[i]; 
			let obs_codes = &header.obs_codes
				.as_ref()
					.unwrap()
					[&sv.constellation];
			
			let mut code_index : usize = 0;
			loop { // per obs code
				let code = &obs_codes[code_index];
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

				let ssi : Option<u8> = match line.len() < offset+14+2 {
					true => {
						// can't parse ssi here
						// 	* line is over and this measurement
						//    does not have ssi 
						None
					},
					false => {
						let ssi = &line[offset+14+1..offset+14+2];
						let ssi = match u8::from_str_radix(&ssi, 10) {
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
				'S' => constellation::Constellation::Sbas,
				_ => return Err(
					RecordError::ParseSvError(
						record::ParseSvError::UnidentifiedConstellation(identifier))),
			};
			let sv = Sv::new(constell, prn);
			// retrieve obs code for that system
			let obs_codes = &header.obs_codes
				.as_ref()
					.unwrap()
					[&sv.constellation];
			let mut offset : usize = 0;
			let mut code_index : usize = 0;
			let mut obs_map : HashMap<String, ObservationData> = HashMap::new();
			loop { // per obs code
				let code = &obs_codes[code_index];
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
				let ssi : Option<u8> = match rem.len() < offset+14+2 {
					true => {
						// can't parse ssi here,
						// line is terminated by an OBS without ssi
						None
					},
					false => {
						let ssi = &rem[offset+14+1..offset+14+2];
						let ssi = match u8::from_str_radix(&ssi, 10) {
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

#[derive(EnumString)]
pub enum CarrierFrequency {
    /// L1 is a GPS/QZSS/Sbas carrier
    L1, 
    /// L2 is a GPS/QZSS carrier
    L2,
    /// L5 is a GPS/QZSS/Sbas/IRNSS carrier
    L5,
    /// L6 is a QZSS carrier
    L6,
    /// S is a IRNSS carrier
    S,
    /// E1 is a Galileo carrier
    E1,
    /// E5a is a Galileo carrier
    E5a,
    /// E5b is a Galileo carrier
    E5b,
    /// E5(E5a+E5b) is a Galileo carrier
    E5,
    /// E6 is a Galileo carrier
    E6,
    /// B1 is a Beidou carrier
    B1,
    /// B1c is a Beidou carrier
    B1c,
    /// B1a is a Beidou carrier
    B1a,
    /// B2a is a Beidou carrier
    B2a,
    /// B2b is a Beidou carrier
    B2b,
    /// B2(B2a+B2b) is a Beidou carrier
    B2,
    /// B3 is a Beidou carrier
    B3,
    /// B3a is a Beidou carrier
    B3a,
    /// G1 is a Glonass channel,
    G1(f64),
    /// G1a is a Glonass channel,
    G1a,
    /// G2 is a Glonass channel,
    G2(f64),
    /// G2a is a Glonass channel,
    G2a,
    /// G3 is a Glonass channel,
    G3,
}

impl CarrierFrequency {
    /// Returns carrier frequency [MHz]
    pub fn frequency (&self) -> f64 {
        match self {
            CarrierFrequency::L1 => 1575.42_f64,
            CarrierFrequency::L2 => 1227.60_f64,
            CarrierFrequency::L5 => 1176.45_f64,
            CarrierFrequency::L6 => 1278.75_f64,
            CarrierFrequency::S => 2492.028_f64,
            CarrierFrequency::E1 => 1575.42_f64,
            CarrierFrequency::E5a => 1176.45_f64,
            CarrierFrequency::E5b => 1207.140_f64,
            CarrierFrequency::E5 => 1191.795_f64,
            CarrierFrequency::E6 => 1278.75_f64,
            CarrierFrequency::B1 => 1561.098_f64,
            CarrierFrequency::B1c => 1575.42_f64,  
            CarrierFrequency::B1a => 1575.42_f64, 
            CarrierFrequency::B2a => 1176.45_f64, 
            CarrierFrequency::B2b => 1207.140_f64, 
            CarrierFrequency::B2 => 1191.795_f64, 
            CarrierFrequency::B3 => 1268.52_f64,
            CarrierFrequency::B3a => 1268.52_f64,
            CarrierFrequency::G1(c) => 1602.0_f64 + c *9.0/16.0, 
            CarrierFrequency::G1a => 1600.995_f64,
            CarrierFrequency::G2(c) => 1246.06_f64 + c* 7.0/16.0,
            CarrierFrequency::G2a => 1248.06_f64, 
            CarrierFrequency::G3 => 1202.025_f64,
        }
    }
}

/*
pub enum SignalStrength {
    DbHz12, // < 12 dBc/Hz
    DbHz12_17, // 12 <= x < 17 dBc/Hz
    DbHz18_23, // 18 <= x < 23 dBc/Hz
    DbHz21_29, // 24 <= x < 29 dBc/Hz
    DbHz30_35, // 30 <= x < 35 dBc/Hz
    DbHz36_41, // 36 <= x < 41 dBc/Hz
    DbHz42_47, // 42 <= x < 47 dBc/Hz
    DbHz48_53, // 48 <= x < 53 dBc/Hz
    DbHz54, // >= 54 dBc/Hz 
}

impl SignalStrength {
    from f64::
}

/// `ObservationCode` related errors
#[derive(Error, Debug)]
pub enum ObservationCodeError {
    #[error("code not recognized \"{0}\"")]
    UnknownObsCode(String),
}

/// Describes different kind of `Observations`
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ObservationCode {
    /// Carrier phase range from antenna
    /// to Sv measured in whole cycles.    
    // 5.2.13
    /// Phase observations between epochs
    /// must be connected by including # integer cycles.   
    /// Phase obs. must be corrected for phase shifts
    PhaseCode,
    /// Positive doppler means Sv is approaching
    DopplerCode,
    /// Pseudo Range is distance (m) from the
    /// receiver antenna to the Sv antenna,
    /// including clock offsets and other biases
    /// such as delays induced by atmosphere
    PseudoRangeCode,
    /// Carrier signal strength observation
    SigStrengthCode,
}

impl Default for ObservationCode {
    fn default() -> ObservationCode { ObservationCode::PseudoRangeCode }
}

impl std::str::FromStr for ObservationCode {
    type Err = ObservationCodeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if is_pseudo_range_obs_code!(s) {
            Ok(ObservationCode::PseudoRangeCode)
        } else if is_phase_carrier_obs_code!(s) {
            Ok(ObservationCode::PhaseCode)
        } else if is_doppler_obs_code!(s) {
            Ok(ObservationCode::DopplerCode)
        } else if is_sig_strength_obs_code!(s) {
            Ok(ObservationCode::SigStrengthCode)
        } else {
            Err(ObservationCodeError::UnknownObsCode(s.to_string()))
        }
    }
} */

mod test {
    use super::*;
    #[test]
    /// Tests `CarrierFrequency` constructor
    fn test_carrier_frequency() {
        assert_eq!(CarrierFrequency::from_str("L1").is_err(),  false);
        assert_eq!(CarrierFrequency::from_str("E5a").is_err(), false);
        assert_eq!(CarrierFrequency::from_str("E7").is_err(),  true);
        assert_eq!(CarrierFrequency::from_str("L1").unwrap().frequency(), 1575.42_f64);
        assert_eq!(CarrierFrequency::from_str("G1a").unwrap().frequency(), 1600.995_f64);
    }
}
