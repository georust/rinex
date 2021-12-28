//! Navigation.rs
//! to describe `Rinex` file body content
//! for NavigationMessage files
use thiserror::Error;
use chrono::Timelike;
use std::str::FromStr;

use crate::version;
use crate::constellation;

/// Describes NAV records specific errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to identify constellation")]
    ParseConstellationError(#[from] constellation::ConstellationError),
}

#[derive(Debug)]
/// `NavigationRecordType` describes type of record
/// for NAV files
pub enum NavigationRecordType {
    Ephemeride,
}

impl Default for NavigationRecordType {
    /// Builds a default `NavigationRecordType`
    fn default() -> NavigationRecordType {
        NavigationRecordType::Ephemeride
    }
}

impl std::str::FromStr for NavigationRecordType {
    type Err = std::num::ParseIntError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.contains("LNAV") {
            Ok(NavigationRecordType::Ephemeride)
        } else {
            Ok(NavigationRecordType::Ephemeride)
        }
    }
}

#[derive(Debug)]
/// `NavigationMsgType`
/// describes messages type for NAV files
pub enum NavigationMsgType {
    Lnav,
    Cnav,
    Cnav2,
    Fdma,
    Inav,
    Fnav,
}

impl Default for NavigationMsgType {
    /// Builds a default `NavigationMsgType`
    fn default() -> NavigationMsgType {
        NavigationMsgType::Cnav
    }
}

impl std::str::FromStr for NavigationMsgType {
    type Err = std::num::ParseIntError;
    /// Builds a `NavigationMsgType` from a string
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.contains("LNAV") {
            Ok(NavigationMsgType::Lnav)
        } else if s.contains("CNAV") {
            Ok(NavigationMsgType::Cnav) 
        } else if s.contains("CNAV2") {
            Ok(NavigationMsgType::Cnav2)
        } else if s.contains("INAV") {
            Ok(NavigationMsgType::Inav)
        } else if s.contains("FNAV") {
            Ok(NavigationMsgType::Fnav)
        } else if s.contains("FDMA") {
            Ok(NavigationMsgType::Fdma)
        } else {
            Ok(NavigationMsgType::Cnav)
        }
    }
}

/// Maximal nb of `Orbits` to this day (V4)
pub const MaxOrbitCount: usize = 9;

/// `Orbit` quadruplet is the main
/// payload of a NAV record
type Orbit = (f64, f64, f64, f64);

/// `NavigationRecord` describes a NAV message frame.   
/// constellation: GNSS for this particular frame,
///       identical accross entire file for unique RINEX NAV files.   
/// sv_id: Sat. Vehicule ID#   
/// epoch: epoch time stamp    
/// sv_clock_bias: (s)   
/// sv_clock_drift: (s.s⁻¹)   
/// sv_clock_drift_rate: (s.s⁻²)
#[derive(Debug)]
pub struct NavigationRecord {
    record_type: NavigationRecordType,
    msg_type: NavigationMsgType,
    constellation: constellation::Constellation,
    sv_id: u8, // Vehicule #ID 
    epoch: chrono::NaiveDateTime, // timestamp
    sv_clock_bias: f64, // (s)
    sv_clock_drift: f64, // (s.s⁻¹)
    sv_clock_drift_rate: f64, // (s.s⁻²)
    orbits: Vec<Orbit>, // orbits (constellation + vers. dependent)
}

impl Default for NavigationRecord {
    fn default() -> NavigationRecord {
        NavigationRecord {
            record_type: NavigationRecordType::default(),
            msg_type: NavigationMsgType::default(),
            constellation: constellation::Constellation::default(),
            sv_id: 0,
            epoch: chrono::NaiveDate::from_ymd(2000,01,01)
                .and_hms(0,0,0),
            sv_clock_bias: 0.0_f64,    
            sv_clock_drift: 0.0_f64,    
            sv_clock_drift_rate: 0.0_f64,    
            orbits: Vec::with_capacity(MaxOrbitCount),
        }
    }
}

impl NavigationRecord {
    /// Builds date (timestamp) from raw str items
    fn parse_date (items: &[&str]) -> Result<chrono::NaiveDateTime, Error> {
        let (mut y,mon,day,h,min,s): (i32,u32,u32,u32,u32,f64) =
            (i32::from_str_radix(items[0], 10)?,
            u32::from_str_radix(items[1], 10)?,
            u32::from_str_radix(items[2], 10)?,
            u32::from_str_radix(items[3], 10)?,
            u32::from_str_radix(items[4], 10)?,
            f64::from_str(items[5])?);
        if y < 100 {
            y += 2000 // 2 digit nb case
        }
        Ok(chrono::NaiveDate::from_ymd(y,mon,day)
            .and_hms(h,min,s as u32))
    }
    
    /// Builds `NavigationRecord` from raw record content
    pub fn from_string (version: &version::Version, constellation: &constellation::Constellation, s: &str) -> Result<NavigationRecord, Error> {
        let mut lines = s.lines();
        let mut line = lines.next()
            .unwrap();

        let mut msg_type = NavigationMsgType::default();
        let mut record_type = NavigationRecordType::default();
        if version.get_major() > 3 {
            let items: Vec<&str> = line.split_ascii_whitespace()
                .collect();
            record_type = NavigationRecordType::from_str(&items[3])?; 
            msg_type = NavigationMsgType::from_str(&items[1])?;
            line = lines.next()
                .unwrap()
        }
        
        // line 1 always contains 
        // [+] SV#ID
        // [+] time stamp
        // [+] sv clock bias
        // [+] sv clock drift
        // [+] sv clock drift rate
        let (sat_id_and_date, rem) = line.split_at(22);
        let (bias, rem) = rem.split_at(19);
        let (drift, rem) = rem.split_at(19);
        let (drift_rate, rem) = rem.split_at(19);

        let items: Vec<&str> = sat_id_and_date.split_ascii_whitespace()
            .collect();

        let nav_message_known_sv_identifiers: &'static [char] =
            &['R','G','E','B','J','C','S']; 

        let (constel, sv_id): (constellation::Constellation, u8) = match nav_message_known_sv_identifiers
                .contains(&items[0].chars().nth(0).unwrap()) 
        {
            true => {
                // V > 3 contains satid#PRN, and not PRN only
                (constellation::Constellation::from_str(items[0])?,
                u8::from_str_radix(&items[0][1..], 10)?)
            },
            false => {
                match constellation {
                    constellation::Constellation::Glonass => {
                        // Glonass dedicated NAV + old fashion: no 'G'
                        (constellation::Constellation::from_str(items[0])?,
                        u8::from_str_radix(&items[0], 10)?)
                    },
                    constellation::Constellation::Mixed => {
                        // Mixed requires 'ID' + PRN#
                        (constellation::Constellation::from_str(items[0])?,
                        u8::from_str_radix(&items[0][1..], 10)?)
                    },
                    c => (*c, u8::from_str_radix(&items[0], 10)?)
                }
            }
        };

        let epoch = NavigationRecord::parse_date(&items[1..7])?;
        println!("BIAS \"{}\" DRIFT \"{}\" RATE \"{}\"",&bias.replace("D","e"),drift,drift_rate);
        let sv_clock_bias = f64::from_str(&bias.replace("D","e"))?;
        let sv_clock_drift = f64::from_str(&drift.replace("D","e"))?;
        let sv_clock_drift_rate = f64::from_str(&drift_rate.replace("D","e"))?;

        // orbits parsing
        loop {
            if let Some(l) = lines.next() {
                line = l;   
            } else {
                break
            }
            let (data1, rem) = line.split_at(23);
            let (data2, rem) = rem.split_at(19);
            let (data3, rem) = rem.split_at(19);
            let (data4, rem) = rem.split_at(19);
        }
        
        Ok(NavigationRecord {
            record_type: NavigationRecordType::default(),
            msg_type: NavigationMsgType::default(),
            constellation: constel,
            sv_id,
            epoch,
            sv_clock_bias,
            sv_clock_drift,
            sv_clock_drift_rate,
            orbits: Vec::new(),
        })
    }
}
/* NAV TIPS */
/*2.10           N: GPS NAV DATA                         RINEX VERSION / TYPE 
 6 99  9  2 17 51 44.0 -.839701388031D-03 -.165982783074D-10  .000000000000D+00
     .910000000000D+02  .934062500000D+02  .116040547840D-08  .162092304801D+00
     .484101474285D-05  .626740418375D-02  .652112066746D-05  .515365489006D+04
     .409904000000D+06 -.242143869400D-07  .329237003460D+00 -.596046447754D-07
     .111541663136D+01  .326593750000D+03  .206958726335D+01 -.638312302555D-08
     .307155651409D-09  .000000000000D+00  .102500000000D+04  .000000000000D+00
     .000000000000D+00  .000000000000D+00  .000000000000D+00  .910000000000D+02
     .406800000000D+06  .000000000000D+00

*/
/* 2.11 GLONASS
 3 20 12 31 23 45  0.0 2.833176404238D-05 0.000000000000D+00 8.637000000000D+04
    1.997111425781D+04 1.119024276733D+00 2.793967723846D-09 0.000000000000D+00
    1.218920263672D+04 8.536128997803D-01 0.000000000000D+00 5.000000000000D+00
   -1.019199707031D+04 3.197331428528D+00 3.725290298462D-09 0.000000000000D+00 */
/* 3.04 mixed
C05 2021 01 01 01 00 00 -.426608719863e-03 -.753255235963e-10  .000000000000e+00
      .100000000000e+01  .546406250000e+03 -.317120352193e-08 -.229379345181e+01
      .176201574504e-04  .403049285524e-03  .226250849664e-04  .649346417427e+04
      .435600000000e+06 -.838190317154e-07 -.243623499101e+01 -.269617885351e-06
      .679641881470e-01 -.690546875000e+03 -.178791507361e+00  .417731685916e-08
     -.869679082767e-09  .000000000000e+00  .782000000000e+03  .000000000000e+00
      .200000000000e+01  .000000000000e+00 -.599999994133e-09 -.900000000000e-08
      .435600000000e+06  .000000000000e+00 0.000000000000e+00 0.000000000000e+00 */
/* 3.0.4 mixed
C01 2021 01 01 01 00 00-7.189651951194e-04 3.441336104970e-11 0.000000000000e+00
     1.000000000000e+00 3.281250000000e-01 7.293160932284e-10 5.910882287859e-01
    -7.217749953270e-08 7.766250055283e-04 2.317829057574e-05 6.493505041122e+03
     4.356000000000e+05 4.703179001808e-08-2.767460601922e+00-1.769512891769e-08
     9.668334299789e-02-7.031718750000e+02-1.235808222247e+00 3.735869899691e-10
    -7.439595603304e-10 0.000000000000e+00 7.820000000000e+02 0.000000000000e+00
     2.000000000000e+00 0.000000000000e+00-5.400000000000e-09-9.900000000000e-09
     4.356000000000e+05 0.000000000000e+00 */
