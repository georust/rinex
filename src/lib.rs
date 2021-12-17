//! RINEX library to parse and analyze RINEX files.
//! Current supported RINEX Version is 2.10.
//! Supported the following RINEX files format
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

/// Describes all known `GNSS constellations`
#[derive(Clone, PartialEq, Debug)]
pub enum Constellation {
    GPS,
    Glonass,
    Beidou,
    QZSS,
    Galileo,
    Mixed, // mixed constellation records
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
        } else if s.starts_with("M") {
            Ok(Constellation::Mixed)
        } else {
            Err(ConstellationError::UnknownConstellation(s.to_string()))
        }
    }
}

/// Describes all known RINEX file types
#[derive(Debug)]
enum DataType {
    Observation,
/*
    Navigation,
    Meteo,
    MeteoData,
    GpsEphemeris,
    GloEphemeris,
    GalEphemeris,
*/
}

#[derive(Error, Debug)]
enum DataTypeError {
    #[error("Unknown RINEX data '{0}'")]
    UnknownDataType(String),
}

impl std::str::FromStr for DataType {
    type Err = DataTypeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("OBSERVATION DATA") {
            Ok(DataType::Observation)
        } else {
            Err(DataTypeError::UnknownDataType(String::from(s)))
        }
    }
}

#[derive(Error, Debug)]
enum HeaderError {
    #[error("RINEX version is not supported '{0}'")]
    VersionNotSupported(String),
    #[error("Non supported header format")]
    FmtError,
    #[error("Unknown RINEX file type '{0}'")]
    UnknownRinexType(String),
    #[error("Unknown GNSS Constellation '{0}'")]
    UnknownConstellation(#[from] ConstellationError),
    #[error("Unknown Data Type '{0}'")]
    DataTypeError(#[from] DataTypeError),
}

/// GNSS receiver description
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
#[derive(Debug)]
struct Antenna {
    model: String,
    sn: String, // serial #
    coords: geo_types::Point<f32>, // ANT approx. coordinates
}

/// GnssTime struct is a `UTC` time 
/// realized from given associated `GNSS constellation`
#[derive(Debug)]
struct GnssTime {
    utc: chrono::DateTime<chrono::Utc>, /// UTC time
    gnss: Constellation,
}

/// Describes RINEX file header
#[derive(Debug)]
struct Header {
    version: String, // Rinex format version
    data: DataType, // file format (observation, data..)
    gnss: Constellation, // GNSS constellation being used
    marker_name: &'static str, // marker name 
    marker_number: &'static str, // marker number
    //date: strtime, // file date of creation
    station: Option<&'static str>, // station label
    observer: &'static str, // observer label
    agency: &'static str, // observer/agency
    rcvr: Rcvr, // receiver used for this recording
    ant: Antenna, // antenna used for this recording
    coords: geo_types::Point<f32>, // station approx. coords
    wavelengths: Option<(u32,u32)>, // L1/L2 wavelengths
    nb_observations: u64,
    //observations: Observation,
    sampling_interval: f32, // sampling
    first_epoch: GnssTime, // date of first observation
    last_epoch: Option<GnssTime>, // date of last observation
    comments: Option<&'static str>, // optionnal comments
    epoch_corrected: bool, // true if epochs are already corrected
    gps_utc_delta: Option<u32>, // optionnal GPS / UTC time difference
    sat_number: Option<u32>, // nb of sat for which we have data
}

/// Header displayer
impl std::fmt::Display for Header {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "RINEX Version: {}",
            self.version
        )
    }
}

impl std::str::FromStr for Header {
    type Err = HeaderError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        let (version, fmt1, fmt2, gnss) = match scan_fmt!(s, 
            "{[0-9].[0-9][0-9]} {} {} {}", String, String, String, String)
        {
            (Some(version), Some(fmt1), Some(fmt2), Some(gnss)) => (version,fmt1,fmt2,gnss),
            _ => return Err(HeaderError::FmtError),
        };

        Ok(Header{
            version,
            data: DataType::from_str(&(String::from(&fmt1).to_owned() + &fmt2))?,
            gnss: Constellation::from_str(&gnss)?,
            marker_name: "",
            marker_number: "",
            station: None,
            observer: "",
            agency: "",
            rcvr: Rcvr {
                model: String::from("test"),
                sn: String::from("test"),
                firmware: String::from("test"),
            },
            ant: Antenna {
                model: String::from("test"),
                sn: String::from("test"),
                coords: geo_types::Point::new(0.0, 0.0),
            },
            coords: geo_types::Point::new(0.0,0.0),
            wavelengths: None,
            nb_observations: 0,
            sampling_interval: 0.0,
            first_epoch: GnssTime {
                utc: chrono::Utc::now(),
                gnss: Constellation::GPS,
            },
            last_epoch: None,
            epoch_corrected: false,
            comments: None,
            gps_utc_delta: None,
            sat_number: None,
        })
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
    /// tests version support identification tool
    fn test_version_tool() {
        assert_eq!(version_is_supported("a.b").is_err(), true); // fmt error
        assert_eq!(version_is_supported("1.0").unwrap(), true); // OK basic
        assert_eq!(version_is_supported("1.0").unwrap(), true); // OK old
        assert_eq!(version_is_supported(VERSION).unwrap(), true); // OK current 
        assert_eq!(version_is_supported("4.0").unwrap(), false); // NOK too recent 
    }

    #[test]
    /// tests Rcvr object fromStr method
    fn rcvr_from_str() {
        /* standard format #1 */
        assert_eq!(
            Rcvr::from_str("82205               LEICA RS500         4.20/1.39")
                .unwrap(),
            Rcvr{
                sn: String::from("82205"),
                model: String::from("LEICA RS500"),
                firmware: String::from("4.20/1.39")
            });
        
        /* faulty whitespaces but passes */
        assert_eq!(
            Rcvr::from_str("82205 LEICA RS500 4.20/1.39")
                .unwrap(),
            Rcvr{
                sn: String::from("82205"),
                model: String::from("LEICA RS500"),
                firmware: String::from("4.20/1.39")
            });
    }

    #[test]
    /// tests Header fromStr method
    fn header_from_str() {
        // should crash triple digit
        assert_eq!(
            Header::from_str("2.100 OBSERVATION DATA G (GPS)").is_err(),
            true);
        // should crash missing .XX
        assert_eq!(
            Header::from_str("2. OBSERVATION DATA G (GPS)").is_err(),
            true);
        // should crash missing .XX
        assert_eq!(
            Header::from_str("2 OBSERVATION DATA G (GPS)").is_err(),
            true);
        // should pass
        let hd = Header::from_str("2.00 OBSERVATION DATA G (GPS)");
        println!("{:?}", hd);
    }
}
