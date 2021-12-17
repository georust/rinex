//! This package provides a set of tools to parse 
//! and analyze RINEX files.
//! 
//! Current supported RINEX Version is 2.11.
//! 
//! The lib is not sensitive to white spaces, whether they
//! be trailing or missing whitespaces. Therefore
//! the lib would accept files that do not respect standard
//! specifications.
//!
//! The lib does not care about end of line description
//! that is most of the time integrated to the header section.
//! exceptions: ?
//!
//! url:

use regex::Regex;
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
    ObservationData,
    NavigationData,
    MeteoData
}

#[derive(Error, Debug)]
enum DataTypeError {
    #[error("Unknown RINEX data '{0}'")]
    UnknownDataType(String),
}

impl std::str::FromStr for DataType {
    type Err = DataTypeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("OBSERVATION") {
            Ok(DataType::ObservationData)
        } else if s.eq("NAVIGATION") {
            Ok(DataType::NavigationData)
        } else if s.eq("METEOROLOGICAL") {
            Ok(DataType::MeteoData)
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
    FormatError,
    #[error("Version string does not match X.YY format")]
    VersionFormatError(#[from] VersionFormatError),
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
    // TODO @GBR
    // use regex here too plz
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
    constellation: Constellation, // GNSS constellation being used
    program: String, // `PGM` program name 
    run_by: String, // marker number
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

/* NOTES
 * The RINEX VERSION / TYPE record must be the first record in a file.
 * • The PGM / RUN BY / DATE line must be the second record(line) in all RINEX
 * files. In RINEX Observation files additional records of this type from previous file
 * modifications or updates can be stored if needed as the lines immediately following
 * the second line.
 * • The SYS / # / OBS TYPES record(s) should precede any SYS / DCBS
 * APPLIED and SYS / SCALE FACTOR records.
 * • The # OF SATELLITES record (if present) should be immediately followed by the
 * corresponding number of PRN / # OF OBS records.
*/
impl std::str::FromStr for Header {
    type Err = HeaderError;
    /// expects all header content as str reference
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        // work on EOL basis
        let lines: Vec<&str> = content.split_terminator('\n').collect(); 
        // line #1 always expected 
        let line = lines.get(0)
            .unwrap();
        // X.YY (data type) 'DATA' [A-Z] (xxnot cared)
     //2.11           G: GLONASS NAV DATA                     RINEX VERSION / TYPE
     //2.11           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE
        let re = Regex::new(r"(\d\.\d{2}) (OBSERVATION|NAVIGATION) DATA [A-Z]")
            .unwrap();
        match re.is_match(line) {
            false => return Err(HeaderError::FormatError),
            _ => {},
        }

        /* Version X.YY verification */
        let mut items = line.split_whitespace(); 
        let version = items.next().unwrap();
        match version_is_supported(version) {
            Ok(true) => {},
            Ok(false) => return Err(HeaderError::VersionNotSupported(version.to_string())),
            Err(e) => return Err(HeaderError::VersionFormatError(e)),
        }

        let data_type = items.next().unwrap();
        let _ = items.next().unwrap(); // discard 'DATA'
        let constellation = items.next().unwrap(); // 'G/M.. constellation descriptor

        // line #2 always expected 
        let line = lines.get(1).unwrap();
        // `Pgm` [str], `Run By` [str], `Date` [yyyymmdd hhmmss], `TZ`
        let re = Regex::new(r"{} {} {}")
            .unwrap();

        let mut items = line.split_whitespace(); 
        let pgm = items.next().unwrap();
        let run_by = items.next().unwrap();
        let date = items.next().unwrap();
        let tz = items.next().unwrap();

        // starting from there we have several options

        Ok(Header{
            version: version.to_string(),
            data: DataType::from_str(data_type)?,
            constellation: Constellation::from_str(constellation)?,
            program: pgm.to_string(),
            run_by: run_by.to_string(),
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

/// Checks whether this lib supports the given RINEX revision number
/// Revision number matches expected format already
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

/// `Rinex` main work structure
/// describes a RINEX file
#[derive(Debug)]
struct Rinex {
    header: Header,
}

#[derive(Error, Debug)]
enum RinexError {
    FileNamingConvention,
    MissingHeaderDelimiter,
    UnknownFileFormat,
    #[error("Header parsing error")]
    HeaderError(#[from] HeaderError), 
}

impl Rinex {

    /// grabs header content of given file
    fn grab_header (fp: &std::path::Path) -> Result<String, RinexError> {
        let content: String = std::fs::read_to_string(fp)
            .unwrap()
                .parse()
                .unwrap();
        if !content.contains("END OF HEADER") {
            return Err(RinexError::MissingHeaderDelimiter)
        }
        let parsed: Vec<&str> = content.split_inclusive("END OF HEADER")
            .collect();
        Ok(parsed[0].to_string())
    }

    /// builds `Rinex` from observation file 
    /// implementation for .o files
    fn from_observation_file (fp: &std::path::Path) -> Result<Rinex, RinexError> {
        let header_str = Rinex::grab_header(fp)?;
        let header = Header::from_str(&header_str)?;
        Err(RinexError::MissingHeaderDelimiter)
    }

    /// builds `Rinex` from GPS nav
    /// implementation for .n files
    //fn from_gps_navigation_file (fp: &std::path::Path) -> Result<Rinex, RinexError> {
    //    let header_content = Rinex::grab_header(fp);
    //    Err(RinexError::RinexE)
    //}

    /// builds `Rinex` from GLO navigation 
    /// implementation for .g files
    //fn from_glonass_navigation_file (fp: &std::path::Path) -> Result<Rinex, RinexError> {
    //    Err(RinexError::RinexE) 
    //}

    /// `Rinex` constructor
    pub fn from (fp: &std::path::Path) -> Result<Rinex, RinexError> {
        let extension = fp.extension()
            .unwrap();
        let extension = extension.to_str()
            .unwrap();
        let extension_re = Regex::new(r"[a-z][a-z][o|m|n|g|l|h|b|c|s]")
            .unwrap();
        if extension_re.is_match(extension) {
            return Err(RinexError::FileNamingConvention)
        }

        if extension.ends_with("o") {
            Rinex::from_observation_file(fp)
        } else {
            //
            // M meteo data file
            // G glonass file
            // L future Gal file
            // H geostationnary GPS payload nav message
            // B Geo SBAS
            // C: clock file
            // S: Summary file
            Err(RinexError::UnknownFileFormat)
        }
    }
}

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
            Rcvr::from_str("82205 LEICA RS500 4.20/1.39")
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
    /// Test `Rinex` constructor
    /// against all valid data resources
    fn rinex_constructor() {
        // open test resources
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data");
        // walk test resources
        for entry in std::fs::read_dir(test_resources)
            .unwrap() {
            let entry = entry
                .unwrap();
            let path = entry.path();
            if !path.is_dir() { // only files..
                let fp = std::path::Path::new(&path);
                assert_eq!(
                    Rinex::from(&fp).is_err(),
                    false,
                    "Rinex::from() failed for '{:?}' with '{:?}'",
                    path, 
                    Rinex::from(&fp))
            }
        }
    }
/*
                let file_name = path.to_str()
                    .unwrap_or("");
                // grab file content
                let content: String = std::fs::read_to_string(file_name)
                    .unwrap()
                        .parse()
                        .unwrap();
                // focus on header content only
                let parsed: Vec<&str> = content.split_inclusive("END OF HEADER").collect();
                let header = parsed[0];
                assert_eq!(
                    Header::from_str(header).is_err(),
                    false,
                    "header::from_str test failed for '{}' with '{:?}'",
                    file_name, Header::from_str(header))
            }
        }
*/
}
