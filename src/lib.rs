//! This package provides a set of tools to parse 
//! and analyze RINEX files.
//! 
//! Current supported RINEX Version is 2.11.
//! 
//! Homepage: <https://github.com/gwbres/rinex>
//!
//! The lib is not sensitive to white spaces, whether they
//! be trailing or missing whitespaces. Therefore
//! the lib would accept files that do not strictly respect 
//! RINEX standards in terms of white spaces. 
//!
//! The lib does not care about end of line description
//! that is most of the time integrated to the header section.
//! Exceptions: ?

use regex::Regex;
use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;
extern crate geo_types;

/// Max. `RINEX` version supported
const VERSION: &str = "3.04"; 

/// Special `Compact RINEX` marker 
const COMPACT_RINEX_HEADER: &str = "COMPACT RINEX FORMAT";

/// Version parsing related error(s)
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

#[derive(Error, Debug)]
enum HeaderError {
    #[error("Compact RINEX related content mismatch")]
    CrinexFormatError,
    #[error("RINEX version is not supported '{0}'")]
    VersionNotSupported(String),
    #[error("Non supported header format")]
    FormatError,
    #[error("Version string does not match X.YY format")]
    VersionFormatError(#[from] VersionFormatError),
    #[error("Unknown GNSS Constellation '{0}'")]
    UnknownConstellation(#[from] ConstellationError),
    #[error("Date parsing error")]
    DateParsingError(#[from] chrono::ParseError),
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

/// Describes `Compact RINEX` specific information
#[derive(Debug)]
struct CrinexInfo {
    version: String,
    prog: String,
    date: chrono::NaiveDateTime,
}

/// Describes `RINEX` file header
#[derive(Debug)]
struct Header {
    version: String, // version description
    crinex: Option<CrinexInfo>, // if this is a CRINEX
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
    epoch_corrected: bool, // true if epochs are already corrected
    comments: Option<String>, // optionnal header comments
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
    /// Builds header from extracted header description
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        let mut content_split: Vec<&str> = content.split_terminator("\n").collect();
        let mut lines = content_split.iter_mut();
        let line = lines.next()
            .unwrap();
        // is a 'compact rinex'?
        let is_crinex = line.contains(COMPACT_RINEX_HEADER);
        let crinex_infos: Option<CrinexInfo> = match is_crinex {
            false => None,
            true => {
                let version = match scan_fmt!(&line, "{}", String) {
                    (Some(version)) => version,
                    _ => return Err(HeaderError::CrinexFormatError),
                };
                let line = lines.next()
                    .unwrap();
                let (prog, date) = match scan_fmt!(&line, "{} {}", String, String) {
                    (Some(prog),Some(date)) => (prog,date),
                    _ => return Err(HeaderError::CrinexFormatError),
                };
                Some(CrinexInfo {
                    version: version.to_string(),
                    prog: prog.to_string(),
                    date: chrono::NaiveDateTime::parse_from_str(
                        &date,
                        "%d-%b-%y %H:%M"
                    )?,
                })
            }
        };

        let line = match is_crinex {
            true => lines.next()
                .unwrap(),
            false => line
        };
/*
        let (version, constellation) : (String, String) = match data_type {
            DataType::ObservationData => {
                match scan_fmt!(&line,
                    "{} {} {} {}",
                    String, String, String, String
                ) {
                    (Some(version),
                    _, // 'OBSERVATION'
                    _, // 'DATA'
                    Some(constellation)) => (version,constellation),
                    _ => return Err(HeaderError::FormatError),
                }
            }
        };
        
        /* Version X.YY verification */
        match version_is_supported(&version) {
            Ok(false) => return Err(HeaderError::VersionNotSupported(version.to_string())),
            Err(e) => return Err(HeaderError::VersionFormatError(e)),
            _ => {},
        }

        // line #2 always expected 
        let line = lines.get(1)
            .unwrap();
        let (pgm, run_by) : (String, String) = match scan_fmt!(line,
            // TODO@
            // on peut pas attendre forcement 2 string
            // sur le nom ici
            "{} {} {} {} {} {}",
            String, String, String
        ) {
            (Some(pgm),Some(run_by),
        }

        let mut items = line.split_whitespace(); 
        let pgm = items.next().unwrap();
        let run_by = items.next().unwrap();
        let date = items.next().unwrap();
        let tz = items.next().unwrap();

        // starting from there we have several options
*/
        Ok(Header{
            version: String::from("Unknown"), 
            crinex: crinex_infos, 
            constellation: Constellation::GPS,
            program: String::from("test"), 
            run_by: String::from("test"),
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

impl Header {
    /// Returns true if self is a `Compact RINEX`
    pub fn is_crinex (&self) -> bool { self.crinex.is_some() }

    /// Returns `Compact RINEX` version (if any) 
    pub fn get_crinex_version (&self) -> Option<&str> { 
        match &self.crinex {
            Some(crinex) => Some(&crinex.version),
            _ => None,
        }
    }
    /// Returns `Compact RINEX` prog (if any) 
    pub fn get_crinex_prog (&self) -> Option<&str> { 
        match &self.crinex {
            Some(crinex) => Some(&crinex.prog),
            _ => None,
        }
    }
    /// Returns `Compact RINEX` date (if any) 
    pub fn get_crinex_date (&self) -> Option<chrono::NaiveDateTime> { 
        match &self.crinex {
            Some(crinex) => Some(crinex.date),
            _ => None,
        }
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


/// `Rinex` main work structure
/// describes a RINEX file
#[derive(Debug)]
struct Rinex {
    header: Header,
    data_type: DataType,
}

/// Describes all known RINEX file types
#[derive(Debug)]
enum DataType {
    ObservationData,
}

#[derive(Error, Debug)]
enum RinexError {
    #[error("File does not follow naming conventions")]
    FileNamingConvention,
    #[error("Header delimiter not found")]
    MissingHeaderDelimiter,
    #[error("Header parsing error")]
    HeaderError(#[from] HeaderError), 
    #[error("Unknown file format '{0}'")]
    UnknownFileFormat(String),
    #[error("TO BE REMOVED")]
    WorkInProgress,
}

impl Rinex {

    /// grabs header content of given file
    fn parse_header_content (fp: &std::path::Path) -> Result<String, RinexError> {
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

    /// `Rinex` constructor
    pub fn from (fp: &std::path::Path) -> Result<Rinex, RinexError> {
        // ssssdddf.yyt
        // ssss: acronyme de la station
        // ddd: jour de l'annee du premier record
        // f: numero de la session dans le jour avec 0 pour une journee complete
        let extension = fp.extension()
            .unwrap();
        let extension = extension.to_str()
            .unwrap();
        /*
         * TODO @ GBR
         * check for naming convention plz
        println!("'{}'", extension);
         * let extension_re = Regex::new(r"\d\d.")
            .unwrap();
        if extension_re.is_match(extension) {
            return Err(RinexError::FileNamingConvention)
        }*/

        // build header
        let header = Header::from_str(&Rinex::parse_header_content(fp)?)?;
        Ok(Rinex{
            header,
            data_type: DataType::ObservationData,
        })
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
}
