//! TODO
//! 
//! url:

use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;
extern crate geo_types;

/// Max. RINEX version supported
const VERSION: &str = "2.10"; 

/// Version parsing related error
#[derive(Error, Debug)]
enum VersionFormatError {
    #[error("Version string does not match expected format")]
    ParseIntError(#[from] std::num::ParseIntError),
}

/// Describes all known GNSS constellations
#[derive(Clone, PartialEq, Debug)]
pub enum Constellation {
    GPS,
    Glonass,
    Beidou,
    QZSS,
    Galileo,
}

#[derive(Error, Debug)]
pub enum ConstellationError {
    #[error("unknown constellation '{0}'")]
    UnknownConstellation(String),
}

impl std::str::FromStr for Constellation {
    type Err = ConstellationError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("G") {
            Ok(Constellation::GPS)
        } else if s.starts_with("E") {
            Ok(Constellation::Galileo)
        } else if s.starts_with("R") {
            Ok(Constellation::Glonass)
        } else if s.starts_with("J") {
            Ok(Constellation::QZSS)
        } else if s.starts_with("C") {
            Ok(Constellation::Beidou)
        } else {
            Err(ConstellationError::UnknownConstellation(s.to_string()))
        }
    }
}

/// Describes all known Rinex file format 
enum Format {
    Observation,
    Meteo,
    MeteoData,
    GpsEphemeris,
    GloEphemeris,
    GalEphemeris,
}

/// Header Parsing / formatting errors
enum HeaderError {
    HeaderError,
    VersionNotSupported,
    UnknownFormat,
}

/// Receiver used in recording
#[derive(Debug, PartialEq)]
struct Rcvr {
    model: String, 
    sn: String, // serial #
    firmware: String, // firmware #
}

#[derive(Debug)]
enum RcvrError {
    FormatError,
}

impl std::str::FromStr for Rcvr {
    type Err = RcvrError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        match scan_fmt!(s, "{} {} {} {}", String, String, String, String) {
            (Some(sn), Some(maker), Some(model), Some(firmware)) => {
                Ok(Rcvr{model: String::from(maker.to_owned() + " " + &model), sn, firmware})
            }
            _ => {
                Err(RcvrError::FormatError)
            }
        }
    }
}

/// Antenna description
struct Antenna {
    model: &'static str,
    sn: &'static str, // serial #
    coords: geo_types::Point<f32>, // ANT approx. coordinates
}

/// GnssTime: `UTC` time with associated `constellation`
struct GnssTime {
    time: chrono::Utc, /// UTC time
    constellation: Constellation,
}

/// Describes file header
struct RinexHeader {
    version: &'static str, // Rinex format version
    fmt: Format, // file format (observation, data..)
    gnss: Constellation, // GNSS constellation being used
    marker_name: &'static str, // marker name 
    marker_number: &'static str, // marker number
    //date: strtime, // file date of creation
    station: Option<&'static str>, // station label
    observer: &'static str, // observer label
    agency: &'static str, // observer/agency
    rcvr: Rcvr, // receiver used for recording
    ant: Antenna, // ant used for recording
    coords: geo_types::Point<f32>, // station approx. coords
    wavelengths: Option<(u32,u32)>, // L1/L2 wavelengths
    nb_obsverations: u64,
    //observations: Observation,
    sampling_interval: f32, // sampling
    first_epoch: GnssTime, // date of first observation
    last_epoch: GnssTime, // date of last observation
    comments: Option<&'static str>, // optionnal comments
    epoch_corrected: bool, // true if epochs are already corrected
    gps_utc_delta: Option<u32>, // optionnal GPS / UTC time difference
    sat_number: Option<u32>, // nb of sat for which we have data
}

//match scan_fmt! (&content, "RCR = {} {} {d}", String, String,)

/// RinexHeader displayer
impl std::fmt::Display for RinexHeader {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "RINEX Version: {}",
            self.version
        )
    }
}

fn version_is_supported (version: &str) -> Result<bool, VersionFormatError> {
    let supported_digits: Vec<&str> = VERSION.split(".").collect();
    let digit0 = u32::from_str_radix(supported_digits.get(0)
        .unwrap(), 
            10)
            .unwrap();
    let digit1 = u32::from_str_radix(supported_digits.get(1)
        .unwrap(),
            10)
            .unwrap();
    let digits: Vec<&str> = version.split(".").collect();
    let target_digit0 = u32::from_str_radix(digits.get(0)
        .unwrap_or(&"?"), 
            10)?;
    let target_digit1 = u32::from_str_radix(digits.get(1)
        .unwrap_or(&"?"), 
            10)?;
    if target_digit0 > digit0 {
        Ok(false)
    } else {
        if target_digit0 == digit0 {
           if target_digit1 <= digit1 {
                Ok(true)
           } else {
               Ok(false)
            }
        } else {
            Ok(true)
        }
    }
}

// ssssdddf.yyt
// ssss: acronyme de la station
// ddd: jour de l'annee du premier record
// f: numero de la session dans le jour avec 0 pour une journee complete
// yy: aneee 2 digit
// t: type de fichier
//RinexObservation:
//2.10           OBSERVATION DATA    M(MIXED)           RINEX VERSION / TYPE

//impl From<std::Path> for Rinex {
//    fn from (path: std::Path) -> Result<Rinex, Error> {
//        let fmt = match Format::from(path) {
//            Err(e) => return Err(e),
//            Ok(Format::Observation) => {
//                // 
//            },
//            Ok(Format::
//        }
//       
//    }
//}

mod test {
    use super::*;

    #[test]
    fn test_version_tool() {
        assert_eq!(version_is_supported("a.b").is_err(), true); // fmt error
        assert_eq!(version_is_supported("1.0").unwrap(), true); // OK basic
        assert_eq!(version_is_supported("1.0").unwrap(), true); // OK old
        assert_eq!(version_is_supported(VERSION).unwrap(), true); // OK current 
        assert_eq!(version_is_supported("4.0").unwrap(), false); // NOK too recent 
    }

    #[test]
    fn rcvr_from_str() {
        /* correct format #1 */
        let rcvr = Rcvr::from_str("82205 LEICA RS500 4.20/1.39");
        println!("{:?}", rcvr);
        assert_eq!(
            rcvr.unwrap(),
            Rcvr{
                sn: String::from("82205"),
                model: String::from("LEICA RS500"),
                firmware: String::from("4.20/1.39")
            });
    }
}
