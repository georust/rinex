//! This package provides a set of tools to parse 
//! and analyze RINEX files.
//! 
//! This lib is work in progress
//! 
//! Homepage: <https://github.com/gwbres/rinex>

use regex::Regex;
use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;
use chrono::{Timelike, Datelike};
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

impl Default for Constellation {
    fn default() -> Constellation {
        Constellation::GPS
    }
}

#[derive(Error, Debug)]
pub enum ConstellationError {
    #[error("unknown constellation \"{0}\"")]
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

impl std::fmt::Display for Constellation {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Constellation::GPS => fmt.write_str("GPS"),
            Constellation::Glonass => fmt.write_str("GLO"),
            Constellation::Beidou => fmt.write_str("BDS"),
            Constellation::QZSS => fmt.write_str("QZS"),
            Constellation::Galileo => fmt.write_str("GAL"),
            _ => fmt.write_str("M"),
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

impl std::str::FromStr for Rcvr {
    type Err = std::io::Error;
    fn from_str (line: &str) -> Result<Self, Self::Err> {
        let (id, remainder) = line.split_at(20);
        let (make, version) = remainder.split_at(20);
        println!("RCVR | ID \"{}\" | MAKE \"{}\" | VERSION \"{}\"",
            id, make, version);
        Ok(Rcvr::default())
    }
}

/// Antenna description 
#[derive(Debug)]
struct Antenna {
    model: String,
    sn: String, // serial #
}

impl Default for Antenna {
    /// Builds default `Antenna` structure
    fn default() -> Antenna {
        Antenna {
            model: String::from("Unknown"),
            sn: String::from("?"),
        }
    }
}

impl std::str::FromStr for Antenna {
    type Err = std::io::Error;
    fn from_str (line: &str) -> Result<Self, Self::Err> {
        let (id, remainder) = line.split_at(20);
        let (make, remainder) = remainder.trim().split_at(20);
        println!("ANTENNA | ID \"{}\" | MAKE \"{}\"", id, make);
        Ok(Antenna::default())/*{
            model: make,
            id: sn,
            coords:*/
    }
}
/// GnssTime struct is a time realization,
/// and the `GNSS` constellation that produced it
#[derive(Debug)]
struct GnssTime {
    time: chrono::NaiveDateTime,
    gnss: Constellation,
}

impl Default for GnssTime {
    /// Builds default `GnssTime` structure
    fn default() -> GnssTime {
        let now = chrono::Utc::now();
        GnssTime {
            time: chrono::NaiveDate::from_ymd(
                now.date().year(),
                now.date().month(),
                now.date().day(),
            ).and_hms(
                now.time().hour(),
                now.time().minute(),
                now.time().second()
            ),
            gnss: Constellation::GPS,
        }
    }
}

impl GnssTime {
    /// Builds a new `GnssTime` object
    fn new(time: chrono::NaiveDateTime, gnss: Constellation) -> GnssTime {
        GnssTime {
            time, 
            gnss
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

impl Default for RinexType {
    fn default() -> RinexType { RinexType::ObservationData }
}

impl std::str::FromStr for RinexType {
    type Err = RinexTypeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("NAVIGATION DATA") {
            Ok(RinexType::NavigationMessage)
        } else if s.eq("OBSERVATION DATA") {
            Ok(RinexType::ObservationData)
        } else if s.contains("G: GLONASS NAV DATA") {
            Ok(RinexType::NavigationMessage)
        } else {
            Err(RinexTypeError::UnknownType(String::from(s)))
        }
    }
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
    station_id: Option<String>, // station id
    observer: Option<String>, // observer
    agency: Option<String>, // agency
    rcvr: Option<Rcvr>, // receiver used for this recording
    ant: Option<Antenna>, // antenna used for this recording
    leap: Option<u32>, // leap seconds
    coords: Option<geo_types::Point<f32>>, // station approx. coords
    wavelengths: Option<(u32,u32)>, // L1/L2 wavelengths
    nb_observations: u64,
    sampling_interval: Option<f32>, // sampling
    epochs: (Option<GnssTime>, Option<GnssTime>), // first , last observations
    // true if epochs & data compensate for local clock drift 
    rcvr_clock_offset_applied: Option<bool>, 
    gps_utc_delta: Option<u32>, // optionnal GPS / UTC time difference
    sat_number: Option<u32>, // nb of sat for which we have data
}

const RINEX_CRINEX_HEADER_COMMENT : &str = "COMPACT RINEX FORMAT";
const RINEX_HEADER_END_COMMENT    : &str = "END OF HEADER";

#[derive(Error, Debug)]
enum HeaderError {
    #[error("Compact RINEX related content mismatch")]
    CrinexFormatError,
    #[error("RINEX version is not supported '{0}'")]
    VersionNotSupported(String),
    #[error("Line \"{0}\" should begin with Rinex version \"x.yy\"")]
    VersionFormatError(String),
    #[error("rinex type error")]
    RinexTypeError(#[from] RinexTypeError),
    #[error("unknown GNSS Constellation '{0}'")]
    UnknownConstellation(#[from] ConstellationError),
    #[error("failed to parse antenna / receiver infos")]
    AntennaRcvrError(#[from] std::io::Error),
    #[error("failed to parse integer value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse date")]
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
            station_id: None,
            observer: None,
            agency: None,
            rcvr: None, 
            ant: None, 
            coords: None, 
            leap: None,
            rcvr_clock_offset_applied: None,
            wavelengths: None,
            nb_observations: 0,
            sampling_interval: None,
            epochs: (None, None),
            gps_utc_delta: None,
            sat_number: Some(0)
        }
    }
}

impl std::str::FromStr for Header {
    type Err = HeaderError;
    /// Builds header from extracted header description
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        let mut lines = content.lines();
        let mut line = lines.next()
            .unwrap();
        // comments ?
        while is_comment(line) {
            line = lines.next()
                .unwrap()
        }
        // 'compact rinex'?
        let is_crinex = line.contains(RINEX_CRINEX_HEADER_COMMENT);
        let crinex_infos: Option<CrinexInfo> = match is_crinex {
            false => None,
            true => {
                let version = line.split_at(20).0.trim();
                let line = lines.next()
                    .unwrap();
                let (pgm, remainder) = line.split_at(20);
                let (_, remainder) = remainder.split_at(20);
                let date = remainder.split_at(20).0.trim();
                println!("CRINEX: VERSION \"{}\" | PGM \"{}\" | DATE \"{}\"", version.trim(), pgm.trim(), date); 
                Some(CrinexInfo {
                    version: version.trim().to_string(),
                    prog: pgm.trim().to_string(),
                    date: chrono::NaiveDateTime::parse_from_str(date, "%d-%b-%y %H:%M")?
                })
            }
        };
        
        if is_crinex {
            line = lines.next()
                .unwrap()
        }
        // comments ?
        while is_comment(line) {
            line = lines.next()
                .unwrap()
        }

        // line1 {} {} {} // label [VERSION/TYPE/GNSS] 
        let (version, remainder) = line.split_at(20);
        let (rinex_type, remainder) = remainder.trim().split_at(20);
        let (constellation, remainder) = remainder.trim().split_at(20);
        println!("RINEX | VERSION \"{}\" | TYPE \"{}\" | OTHER \"{}\"", version, rinex_type, constellation);

        // version x.yy verification
        match version_is_supported(version.trim()) {
            Ok(false) => return Err(HeaderError::VersionNotSupported(version.to_string())),
            Err(e) => return Err(HeaderError::VersionFormatError(e.to_string())),
            _ => {},
        }
        
        // line2
        line = lines.next()
            .unwrap();
        // comments ?
        while is_comment(line) {
            line = lines.next()
                .unwrap()
        }

        //          | 20      |20
        // {}       {}        yyyymmdd HH:MM:SSUTC
        // {}       {}        yyyymmdd HH:MM:SSLCL
        // {}       {}        yyyymmdd HHMMSS UTC
        // {}                 yyyymmdd HHMMSS LCL
        let (pgm, remainder) = line.split_at(20);
        let (run_by, remainder) = remainder.split_at(20);
        let (date_str, _) = remainder.split_at(20);
        // identify date format (UTC/LCL)
        let is_utc = date_str.contains("UTC");
        let date_str: &str = match is_utc {
            true => {
                let offset = date_str.rfind("UTC")
                    .unwrap();
                date_str.split_at(offset).0.trim() 
            },
            false => {
                let offset = date_str.rfind("LCL")
                    .unwrap();
                date_str.split_at(offset).0.trim() 
            }
        };
        // identify date format (YMDHMS)
        println!("date str \"{}\"", date_str);
        let regex: Vec<Regex> = vec![
            Regex::new(r"\d\d\d\d\d\d\d\d \d\d:\d\d:\d\d$")
                .unwrap(),
            Regex::new(r"\d\d\d\d\d\d\d\d \d\d\d\d\d\d$")
                .unwrap(),
        ];
        let date_fmt: Vec<&str> = vec![
            "%Y%m%d %H:%M:%S",
            "%Y%m%d %H%M%S",
        ];

/*
         * for i in 0..regex.len() {
            if regex[i].is_match(cleanedup) {
                if is_utc {
                   let date = chrono::Utc::parse_from_str(cleanedup)
                } else {
                    let date = chrono::NaiveDate::parse_from_str( 
                }
            }
        }
*/
        line = lines.next()
            .unwrap();
        // comments ?
        while is_comment(line) {
            line = lines.next()
                .unwrap()
        }

        // order may vary from now on
        let mut ant        : Option<Antenna> = None;
        let mut rcvr       : Option<Rcvr>    = None;
        let mut station    : Option<String>  = None;
        let mut station_id : Option<String>  = None;
        let mut observer   : Option<String>  = None;
        let mut agency     : Option<String>  = None;
        let mut leap       : Option<u32>     = None;
        let mut sampling_interval: Option<f32> = None;
        let mut rcvr_clock_offset_applied: Option<bool> = None;
        let mut coords     : Option<geo_types::Point<f32>> = None;
        let mut epochs: (Option<GnssTime>, Option<GnssTime>) = (None, None);
        loop {
            if line.contains("MARKER NAME") {
                station = Some(String::from(line.split_at(20).0.trim()))
            } else if line.contains("MARKER NUMBER") {
                station_id = Some(String::from(line.split_at(20).0.trim())) 
            
            } else if line.contains("OBSERVER / AGENCY") {
                let (obs, ag) = line.split_at(20);
                let ag = ag.split_at(20).0;
                observer = Some(String::from(obs.trim()));
                agency = Some(String::from(ag.trim()))

            } else if line.contains("REC # / TYPE / VERS") {
                rcvr = Some(Rcvr::from_str(line)?) 
            } else if line.contains("ANT # / TYPE") {
                ant = Some(Antenna::from_str(line)?) 
            
            } else if line.contains("LEAP SECOND") {
                let leap_str = line.split_at(20).0.trim();
                leap = Some(u32::from_str_radix(leap_str, 10)?)
            
            } else if line.contains("TIME OF FIRST OBS") {
                let items: Vec<&str> = line.split_ascii_whitespace()
                    .collect();
                let (y, month, d, h, min, s, constel): (i32,u32,u32,u32,u32,f32,Constellation) =
                    (i32::from_str_radix(items[0].trim(),10)?,
                    u32::from_str_radix(items[1].trim(),10)?,
                    u32::from_str_radix(items[2].trim(),10)?,
                    u32::from_str_radix(items[3].trim(),10)?,
                    u32::from_str_radix(items[4].trim(),10)?,
                    f32::from_str(items[5].trim())?,
                    Constellation::from_str(items[6].trim())?);
                let utc = chrono::NaiveDate::from_ymd(y,month,d).and_hms(h,min,s as u32);
                epochs.0 = Some(GnssTime::new(utc, constel)) 

            } else if line.contains("TIME OF LAST OBS") {
                let items: Vec<&str> = line.split_ascii_whitespace()
                    .collect();
                let (y, month, d, h, min, s, constel): (i32,u32,u32,u32,u32,f32,Constellation) =
                    (i32::from_str_radix(items[0].trim(),10)?,
                    u32::from_str_radix(items[1].trim(),10)?,
                    u32::from_str_radix(items[2].trim(),10)?,
                    u32::from_str_radix(items[3].trim(),10)?,
                    u32::from_str_radix(items[4].trim(),10)?,
                    f32::from_str(items[5].trim())?,
                    Constellation::from_str(items[6].trim())?);
                let utc = chrono::NaiveDate::from_ymd(y,month,d).and_hms(h,min,s as u32);
                epochs.1 = Some(GnssTime::new(utc, constel)) 
            
            } else if line.contains("WAVELENGTH FACT L1/2") {
            
            } else if line.contains("APPROX POSITION XYZ") {
                let items: Vec<&str> = line.split_ascii_whitespace()
                    .collect();
                let (x, y, z): (f32,f32,f32) = 
                    (f32::from_str(items[0].trim())?,
                    f32::from_str(items[1].trim())?,
                    f32::from_str(items[2].trim())?);
                coords = Some(geo_types::Point::new(x, y)) // Z! pas le bon objet 

            } else if line.contains("ANTENNA: DELTA H/E/N") {
                //TODO
            } else if line.contains("RCV CLOCK OFFS APPL") {
                let ok_str = line.split_at(20).0.trim();
                rcvr_clock_offset_applied = Some(i32::from_str_radix(ok_str, 10)? != 0)

            } else if line.contains("# OF SATELLITES") {
                //TODO
            } else if line.contains("PRN / # OF OBS") {
                //TODO
            } else if line.contains("SYS / PHASE SHIFT") {
                //TODO
            } else if line.contains("SYS / # / OBS TYPES") {
                //TODO
            } else if line.contains("SYS / PHASE SHIFT") {
                //TODO
            } else if line.contains("SIGNAL STRENGHT UNIT") {
                //TODO
            } else if line.contains("INTERVAL") {
                let intv = line.split_at(20).0.trim();
                sampling_interval = Some(f32::from_str(intv)?)

            } else if line.contains("GLONASS SLOT / FRQ #") {
                //TODO
            } else if line.contains("GLONASS COD/PHS/BIS") {
                //TODO

            } else if line.contains("ION ALPHA") { 
                //TODO
                //0.7451D-08 -0.1490D-07 -0.5960D-07  0.1192D-06          ION ALPHA           

            } else if line.contains("ION BETA") {
                //TODO
                //0.9011D+05 -0.6554D+05 -0.1311D+06  0.4588D+06          ION BETA            
            
            } else if line.contains("DELTA-UTC") {
                //TODO
                //0.931322574615D-09 0.355271367880D-14   233472     1930 DELTA-UTC: A0,A1,T,W
            }

            if let Some(l) = lines.next() {
                line = l
            } else {
                break
            }
            // comments ?
            while is_comment(line) {
                line = lines.next()
                    .unwrap()
            }
            //end_of_header = line.contains("END OF HEADER")
        }
        
        Ok(Header{
            version: version.trim().to_string(),
            crinex: crinex_infos, 
            rinex_type: RinexType::from_str(rinex_type.trim())?,
            constellation: Constellation::from_str(constellation.trim())?, 
            program: String::from(pgm.trim()),
            run_by: Some(String::from(run_by.trim())),
            station: station,
            station_id: station_id,
            agency: agency,
            observer: observer,
            rcvr: rcvr, 
            ant: ant, 
            leap: leap,
            rcvr_clock_offset_applied: rcvr_clock_offset_applied,
            coords: coords,
            wavelengths: None,
            nb_observations: 0,
            sampling_interval: sampling_interval,
            epochs: epochs,
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

    /// Builds a `Rinex` from given file.
    /// Input file must respect the whitespace specifications
    /// for the entire header section.   
    /// The header section must respect the labelization standard too.
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
/*
    /// tests Rcvr object fromStr method
    fn rcvr_from_str() {
        assert_eq!(
            Rcvr::from_str("82205               LEICA RS500         4.20/1.39  ")
                .unwrap(),
            Rcvr{
                sn: String::from("82205"),
                model: String::from("LEICA RS500"),
                firmware: String::from("4.20/1.39")
            });
    }
*/
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
                assert_eq!(rinex.is_err(), false);
                println!("File: {:?}\n{:#?}", &fp, rinex)
            }
        }
    }
}
