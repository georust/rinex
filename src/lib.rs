//! This package provides a set of tools to parse 
//! and analyze RINEX files.
//! 
//! This lib is work in progress, working towards
//! `RINEX` 3.04 support
//! 
//! Homepage: <https://github.com/gwbres/rinex>
//!
//! The lib is not sensitive to white spaces, whether they
//! be trailing or missing whitespaces. Therefore
//! the lib would accept files that do not strictly respect 
//! RINEX standards in terms of white spaces. 

use regex::Regex;
use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;
extern crate geo_types;

/// Current `RINEX` version supported
const SUPPORTED_VERSION: &str = "3.04"; 

/// Checks whether this lib supports the given RINEX revision number
/// Revision number matches expected format already
fn version_is_supported (version: &str) -> Result<bool, std::num::ParseIntError> {
    let supported_digits: Vec<&str> = SUPPORTED_VERSION.split(".").collect();
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

/// Checks whether this (header) line is a comment or not
fn is_comment (line: &str) -> bool { line.contains("COMMENT") }

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

/// GNSS receiver description
#[derive(Debug, PartialEq)]
struct Rcvr {
    model: String, 
    sn: String, // serial #
    firmware: String, // firmware #
}

impl Default for Rcvr {
    /// Builds a `default` Receiver
    fn default() -> Rcvr {
        Rcvr {
            model: String::from("Unknown"),
            sn: String::from("?"),
            firmware: String::from("?"),
        }
    }
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

impl Default for Antenna {
    /// Builds default `Antenna` structure
    fn default() -> Antenna {
        Antenna {
            model: String::from("Unknown"),
            sn: String::from("?"),
            coords: geo_types::Point::<f32>::new(0.0,0.0),
        }
    }
}

/// GnssTime struct is a `UTC` time tied to a 
/// `GNSS` constellation that produced this realization
#[derive(Debug)]
struct GnssTime {
    utc: chrono::DateTime<chrono::Utc>, /// UTC time
    gnss: Constellation,
}

impl Default for GnssTime {
    /// Builds default `GnssTime` structure
    fn default() -> GnssTime {
        GnssTime {
            utc: chrono::Utc::now(),
            gnss: Constellation::GPS,
        }
    }
}

/// Describes `Compact RINEX` specific information
#[derive(Debug)]
struct CrinexInfo {
    version: String, // compression version
    prog: String, // compression program
    date: chrono::NaiveDateTime, // date of compression
}

/// Describes all known RINEX file types
#[derive(Debug)]
enum RinexType {
    ObservationData,
    NavigationMessage,
    MeteorologicalData,
    ClockData,
}

#[derive(Error, Debug)]
enum RinexTypeError {
    #[error("Unknown RINEX type identifier \"{0}\"")]
    UnknownType(String),
}

/// Describes known marker types
enum MarkerType {
    Geodetic, // earth fixed high precision
    NonGeodetic, // earth fixed low prcesion
    NonPhysical, // generated from network
    Spaceborne, // orbiting space vehicule
    Airborne, // aircraft, balloon, ...
    WaterCraft, // mobile water craft
    GroundCraft, // mobile terrestrial vehicule
    FixedBuoy, // fixed on water surface
    FloatingBuoy, // floating on water surface
    FloatingIce, // floating on ice sheet
    Glacier, // fixed on a glacier
    Ballistic, // rockets, shells, etc..
    Animal, // animal carrying a receiver
    Human, // human being
}

impl std::str::FromStr for RinexType {
    type Err = RinexTypeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.contains("OBSERVATION") {
            Ok(RinexType::ObservationData)
        } else if s.contains("NAVIGATION") {
            Ok(RinexType::NavigationMessage)
        } else if s.contains("METEOROLOGICAL") {
            Ok(RinexType::MeteorologicalData)
        } else if s.contains("CLOCK") {
            Ok(RinexType::ClockData)
        } else {
            Err(RinexTypeError::UnknownType(String::from(s)))
        }
    }
}
/// Describes `RINEX` file header
#[derive(Debug)]
struct Header {
    version: String, // version description
    crinex: Option<CrinexInfo>, // if this is a CRINEX
    rinex_type: RinexType, // type of Rinex
    constellation: Constellation, // GNSS constellation being used
    program: String, // program name 
    run_by: Option<String>, // program run by
    //date: strtime, // file date of creation
    station: Option<String>, // station label
    observer: String, // observer label
    agency: String, // observer/agency
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
    gps_utc_delta: Option<u32>, // optionnal GPS / UTC time difference
    sat_number: Option<u32>, // nb of sat for which we have data
}

const RINEX_CRINEX_HEADER_COMMENT : &str = "COMPACT RINEX FORMAT";
const RINEX_HEADER_LINE1_COMMENT  : &str = "RINEX VERSION / TYPE";
const RINEX_HEADER_LINE2_COMMENT  : &str = "PGM / RUN BY / DATE";
const RINEX_HEADER_END_COMMENT    : &str = "END OF HEADER";

#[derive(Error, Debug)]
enum HeaderError {
    #[error("Compact RINEX related content mismatch")]
    CrinexFormatError,
    #[error("RINEX version is not supported '{0}'")]
    VersionNotSupported(String),
    #[error("Line \"{0}\" should begin with Rinex version \"x.yy\"")]
    VersionFormatError(String),
    #[error("Failed to parse Rinex version from line \"{0}\"")]
    VersionParsingError(String),
    #[error("Missing expected label on line #1")]
    MissingRinexVersionTypeComment,
    #[error("Missing expected label on line #2")]
    MissingRinexPgmRunByComment,
    #[error("Failed to identify data type in \"{0}\"")]
    RinexTypeIdentificationError(String),
    #[error("Rine type error")]
    RinexTypeError(#[from] RinexTypeError),
    #[error("Unknown GNSS Constellation '{0}'")]
    UnknownConstellation(#[from] ConstellationError),
    #[error("Failed to parse date")]
    DateParsingError(#[from] chrono::ParseError),
}

impl Default for Header {
    fn default() -> Header {
        Header {
            version: String::from(SUPPORTED_VERSION),
            crinex: None,
            rinex_type: RinexType::ObservationData,
            constellation: Constellation::GPS,
            program: String::from("pgm"),
            run_by: None,
            station: None,
            agency: String::from("me"),
            observer: String::from("myself"),
            rcvr: Rcvr::default(),
            ant: Antenna::default(),
            coords: geo_types::Point::new(0.0, 0.0),
            wavelengths: None,
            nb_observations: 0,
            sampling_interval: 1.0,
            first_epoch: GnssTime::default(),
            last_epoch: None,
            epoch_corrected: false,
            gps_utc_delta: None,
            sat_number: Some(0)
        }
    }
}

/* NOTES
 * • The PGM / RUN BY / DATE line must be the second record(line) in all RINEX
 * files. 
 * <o customization in RINEX Obs described on line #3 ??
 * • The SYS / # / OBS TYPES record(s) should precede any SYS / DCBS
 * APPLIED and SYS / SCALE FACTOR records.
 * • The # OF SATELLITES record (if present) should be immediately followed by the corresponding number of PRN / # OF OBS records.
*/
impl std::str::FromStr for Header {
    type Err = HeaderError;
    /// Builds header from extracted header description
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        let mut content_split: Vec<&str> = content.split_terminator("\n")
            .collect();
        let mut lines = content_split
            .iter_mut();
        let mut line = lines.next()
            .unwrap();
        // comments ?
        while is_comment(line) {
            line = lines.next()
                .unwrap();
        }
        // 'compact rinex'?
        let is_crinex = line.contains(RINEX_CRINEX_HEADER_COMMENT);
        let crinex_infos: Option<CrinexInfo> = match is_crinex {
            false => None,
            true => {
                let version = match scan_fmt!(&line, "{}", String) {
                    Some(version) => version,
                    _ => return Err(HeaderError::CrinexFormatError),
                };
                let line = lines.next()
                    .unwrap();
                let (prog1, prog2, ymd, hm) = match scan_fmt!(&line, "{} ver.{} {} {}", String, String, String, String) {
                    (Some(prog1),Some(prog2),Some(ymd),Some(hm)) => (prog1,prog2,ymd,hm),
                    _ => return Err(HeaderError::CrinexFormatError),
                };
                let prog_str = prog1.to_owned() + " ver." + &prog2;
                let date_str = ymd.to_owned() + &hm;
                Some(CrinexInfo {
                    version: version.to_string(),
                    prog: prog_str,
                    date: chrono::NaiveDateTime::parse_from_str(
                        &date_str,
                        "%d-%b-%y %H:%M"
                    )?,
                })
            }
        };
        
        if is_crinex {
            line = lines.next()
                .unwrap();
            // comments ?
            while is_comment(line) {
                line = lines.next()
                    .unwrap();
            }
        }
        
        // line1
        let cleanedup = String::from(line.trim()); // rm trailing 
        // must match X.YY pattern
        let line1_regex = Regex::new(r"^\d\.\d\d ")
            .unwrap();
        if !line1_regex.is_match(&cleanedup) {
            return Err(HeaderError::VersionFormatError(String::from(cleanedup)))
        }
        // map x.yy
        let version = match scan_fmt!(&cleanedup, "{} ", String) {
            Some(version) => version,
            _ => return Err(HeaderError::VersionParsingError(String::from(cleanedup))),
        };
        
        // version x.yy verification
        match version_is_supported(&version) {
            Ok(false) => return Err(HeaderError::VersionNotSupported(version.to_string())),
            Err(e) => return Err(HeaderError::VersionFormatError(e.to_string())),
            _ => {},
        }

        // rm previously matched version
        let cleanedup = cleanedup.strip_prefix(&version)
            .unwrap(); // already matching..
        // rm end of line label
        let cleanedup = match cleanedup.strip_suffix(RINEX_HEADER_LINE1_COMMENT) {
            Some(s) => s.trim(),
            _ => return Err(HeaderError::MissingRinexVersionTypeComment),
        };
        println!("CLEANED UP \"{}\"", cleanedup);
        // remainder is data type descriptor
        let (rinex_type, constellation): (RinexType, Constellation)
                = match cleanedup.contains("G: GLONASS NAV DATA")
        {
            true => {
                // GLONASS NAV. DATA (.g): special case
                (RinexType::ObservationData,
                Constellation::Glonass)
            },
            false => {
                // nominal case, .d, .o
                // {} {DATA/MAPS} {} (type0, _, constellation)
                match scan_fmt!(&cleanedup, "{} {} {}", String, String, String) {
                    (Some(data),_,Some(constellation)) => {
                        (RinexType::from_str(&data)?,
                        Constellation::from_str(&constellation)?)
                    },
                    _ => return Err(HeaderError::RinexTypeIdentificationError(String::from(cleanedup))),
                }
            },
        };

        // line2
        line = lines.next()
            .unwrap();
        // comments ?
        while is_comment(line) {
            line = lines.next()
                .unwrap();
        }

        // {} {}        yyyymmdd HH:MM:SSUTC
        // {} {}        yyyymmdd HH:MM:SSLCL
        // {} {}        yyyymmdd HHMMSS UTC
        // {}           yyyymmdd HHMMSS UTC
        // 0. remove label cause its most of the time 
        //    right next to UTC/LCL
        println!("{}", line);
        let cleanedup = match line.strip_suffix(RINEX_HEADER_LINE2_COMMENT) {
            Some(s) => s.trim(),
            _ => return Err(HeaderError::MissingRinexPgmRunByComment),
        };
        // 1. start with date cause it's always there
        //    identify date format
        let utc_regex = vec![
            r"\d\d\d\d\d\d\d\d \d\d:\d\d:\d\dUTC$",
            r"\d\d\d\d\d\d\d\d \d\d\d\d\d\dUTC$",
        ];
        let lcl_regex = vec![
            r"\d\d\d\d\d\d\d\d \d\d:\d\d:\d\dLCL$",
            r"\d\d\d\d\d\d\d\d \d\d\d\d\d\dLCL$",
        ];
        //    parse date & remove it

        // 2. then PGM is always there
        //    parse PGM & remove it

        // 3. `run by` is sometimes not provided,
        //    do we have a remainder? 
        
        Ok(Header{
            version,
            crinex: crinex_infos, 
            rinex_type,
            constellation,
            program: String::from("test"), 
            run_by: None, 
            station: None,
            observer: String::from("?"),
            agency: String::from("?"),
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

/// `Rinex` main work structure
/// describes a RINEX file
#[derive(Debug)]
struct Rinex {
    header: Header,
}

#[derive(Error, Debug)]
enum RinexError {
    #[error("File does not follow naming conventions")]
    FileNamingConvention,
    #[error("Header delimiter not found")]
    MissingHeaderDelimiter,
    #[error("Header parsing error")]
    HeaderError(#[from] HeaderError),
}

impl Rinex {
    /// Builds a Rinex struct
    pub fn new (header: Header) -> Rinex {
        Rinex {
            header
        }
    }

    /// grabs header content of given file
    fn parse_header_content (fp: &std::path::Path) -> Result<String, RinexError> {
        let content: String = std::fs::read_to_string(fp)
            .unwrap()
                .parse()
                .unwrap();
        if !content.contains(RINEX_HEADER_END_COMMENT) {
            return Err(RinexError::MissingHeaderDelimiter)
        }
        let parsed: Vec<&str> = content.split(RINEX_HEADER_END_COMMENT)
            .collect();
        Ok(parsed[0].to_string())
    }

    /// `Rinex` constructor
    pub fn from_file (fp: &std::path::Path) -> Result<Rinex, RinexError> {
        let name = fp.file_name()
            .unwrap();
        let extension = fp.extension()
            .unwrap();
        let extension = extension.to_str()
            .unwrap();
        //TODO
        //TODO .s (summary files) not supported 
        // standard pour le nom est
        // ssssdddf.yyt
        // ssss: acronyme de la station
        // ddd: jour de l'annee du premier record
        // f: numero de la session dans le jour avec 0 pour une journee complete
        /*if !extension.eq("crx") && !extension.eq("rnx") {
            // crinex, could have a regex prior "."
            // decompressed crinex ?
            let convention_re = Regex::new(r"\d\d\d\d\.\d\d[o|O|g|G|i|I|d|D|s|S]$")
                .unwrap();
            if !convention_re.is_match(
                name.to_str()
                    .unwrap()) {
                return Err(RinexError::FileNamingConvention)
            }
        }*/

        // build header
        let header = Header::from_str(&Rinex::parse_header_content(fp)?)?;
        Ok(Rinex{
            header
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
        assert_eq!(version_is_supported(SUPPORTED_VERSION).unwrap(), true); // OK current 
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
                let rinex = Rinex::from_file(&fp);
                assert_eq!(
                    rinex.is_err(), 
                    false,
                    "Rinex::from_file() failed for '{:?}' with '{:?}'",
                    path, 
                    rinex);
                println!("File: {:?}\n{:#?}", &fp, rinex)
            }
        }
    }
}
