use regex::Regex;
use thiserror::Error;
use chrono::{Timelike, Datelike};

use crate::version;
use crate::constellation;
use crate::is_rinex_comment;

/// Describes a `CRINEX` (compact rinex) 
pub const CRINEX_MARKER_COMMENT : &str = "COMPACT RINEX FORMAT";
/// End of Header section reached
pub const HEADER_END_MARKER : &str = "END OF HEADER";

/// Describes all known `RINEX` file types
#[derive(Copy, Clone, Debug)]
pub enum RinexType {
    ObservationData,
    NavigationMessage,
    MeteorologicalData,
    ClockData,
}

#[derive(Error, Debug)]
pub enum RinexTypeError {
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
        } else if s.contains("N: GPS NAV DATA") {
            Ok(RinexType::NavigationMessage)
        } else if s.contains("N: GNSS NAV DATA") {
            Ok(RinexType::NavigationMessage)
        } else {
            Err(RinexTypeError::UnknownType(String::from(s)))
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
    coords: Option<rust_3d::Point3D>, // ref. point position
        // for recordings generated onboard a vehicule
    height: Option<f32>, // height in comparison to 
        // reference point (ARP) above marker
    delta_h_e_n: Option<f32>, // horizontal eccentricity (ARP) 
        // relative to marker (EAST/NORTH) 
}

impl Default for Antenna {
    /// Builds default `Antenna` structure
    fn default() -> Antenna {
        Antenna {
            model: String::from("Unknown"),
            sn: String::from("?"),
            coords: None,
            height: None,
            delta_h_e_n: None,
        }
    }
}

/// GnssTime struct is a time realization,
/// and the `GNSS` constellation that produced it
#[derive(Debug)]
struct GnssTime {
    time: chrono::NaiveDateTime,
    gnss: constellation::Constellation,
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
            gnss: constellation::Constellation::default(),
        }
    }
}

impl GnssTime {
    /// Builds a new `GnssTime` object
    fn new(time: chrono::NaiveDateTime, gnss: constellation::Constellation) -> GnssTime {
        GnssTime {
            time, 
            gnss
        }
    }
}

/// `LeapSecond` to describe leap seconds
struct LeapSecond {
    leap: u32, // current amount of leap secs
    week: u32, // week number 
    day: u32,
    delta: u32, // ΔtLSF(BNK) [s]
        // delta time between GPS and UTC due to leap second
        // can be future or past ΔtLSF(BNK) depending
        // wether (week,day) are in future or past
    constellation: constellation::Constellation, // system time identifier
}

impl Default for LeapSecond {
    fn default() -> LeapSecond {
        LeapSecond {
            leap: 0,
            week: 0,
            day: 0,
            delta: 0,
            constellation: constellation::Constellation::default(),
        }
    }
}

impl LeapSecond {
    // Builds a new leap second
    pub fn new (leap: u32, week: Option<u32>, day: Option<u32>, delta: Option<u32>,
        constellation: Option<constellation::Constellation>)
            -> LeapSecond {
        LeapSecond {
            leap: leap,
            week: week.unwrap_or(0),
            day: day.unwrap_or(0),
            delta: delta.unwrap_or(0),
            constellation: constellation.unwrap_or(constellation::Constellation::GPS),
        }
    }
}

impl std::str::FromStr for LeapSecond {
    type Err = HeaderError; 
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ls = LeapSecond::default();
        // leap seconds might have either simple or complex format
        let items: Vec<&str> = s.split_ascii_whitespace()
            .collect();
        match items.len() {
            1 => {
                ls.leap = u32::from_str_radix(items[0].trim(),10)?
            },
            4 => {
                ls.leap = u32::from_str_radix(items[0].trim(),10)?;
                ls.week = u32::from_str_radix(items[1].trim(),10)?;
                ls.day = u32::from_str_radix(items[2].trim(),10)?
                //ls.constellation = Constellation: //TODO
                //18    18  2185     7GPS             LEAP SECONDS        
            },
            _ => return Err(HeaderError::LeapSecondParsingError(String::from(s)))
        }
        Ok(ls)
    }
}

/// Describes `Compact RINEX` specific information
#[derive(Debug)]
struct CrinexInfo {
    version: String, // compression version
    prog: String, // compression program
    date: chrono::NaiveDateTime, // date of compression
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

/// Clock Data types
enum ClockDataType {
    ClockDataAr, // recvr clocks derived from a network
    ClockDataAs, // sat clocks derived from a network
    ClockDataCr, // Calibration measurements for a single GNSS rcvr
    ClockDataDr, // Discontinuity measurements for a single GPS receiver
    ClockDataMs, // Monitor measurements for broadcast sat clocks
}

enum SignalStrength {
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

/// Describes `RINEX` file header
#[derive(Debug)]
pub struct Header {
    version: version::Version, // version description
    crinex: Option<CrinexInfo>, // if this is a CRINEX
    rinex_type: RinexType, // type of Rinex
    constellation: constellation::Constellation, // GNSS constellation being used
    program: String, // program name 
    run_by: String, // program run by
    //date: strtime, // file date of creation
    station: String, // station label
    station_id: String, // station id
    observer: String, // observer
    agency: String, // agency
    rcvr: Option<Rcvr>, // receiver used for this recording
    ant: Option<Antenna>, // optionnal antenna infos
    leap: Option<u32>, // leap seconds
    coords: Option<rust_3d::Point3D>, // station approx. coords
    wavelengths: Option<(u32,u32)>, // L1/L2 wavelengths
    nb_observations: u64,
    sampling_interval: Option<f32>, // sampling
    epochs: (Option<GnssTime>, Option<GnssTime>), // first , last observations
    // true if epochs & data compensate for local clock drift 
    rcvr_clock_offset_applied: Option<bool>, 
    gps_utc_delta: Option<u32>, // optionnal GPS / UTC time difference
    sat_number: Option<u32>, // nb of sat for which we have data
}

#[derive(Error, Debug)]
pub enum HeaderError {
    #[error("Compact RINEX related content mismatch")]
    CrinexFormatError,
    #[error("RINEX version is not supported '{0}'")]
    VersionNotSupported(String),
    #[error("Line \"{0}\" should begin with Rinex version \"x.yy\"")]
    VersionFormatError(String),
    #[error("rinex type error")]
    RinexTypeError(#[from] RinexTypeError),
    #[error("unknown GNSS Constellation '{0}'")]
    UnknownConstellation(#[from] constellation::ConstellationError),
    #[error("failed to parse antenna / receiver infos")]
    AntennaRcvrError(#[from] std::io::Error),
    #[error("failed to parse integer value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse date")]
    DateParsingError(#[from] chrono::ParseError),
    #[error("failed to parse leap second from \"{0}\"")]
    LeapSecondParsingError(String),
}

impl Default for Header {
    fn default() -> Header {
        Header {
            version: version::Version::default(), 
            crinex: None,
            rinex_type: RinexType::default(),
            constellation: constellation::Constellation::default(),
            program: String::from("Unknown"),
            run_by: String::from("Unknown"),
            station: String::from("Unknown"),
            station_id: String::from("Unknown"),
            observer: String::from("Unknown"),
            agency: String::from("Unknown"),
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
        while is_rinex_comment!(line) {
            line = lines.next()
                .unwrap()
        }
        // 'compact rinex'?
        let is_crinex = line.contains(CRINEX_MARKER_COMMENT);
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
        while is_rinex_comment!(line) {
            line = lines.next()
                .unwrap()
        }

        // line1 {} {} {} // label [VERSION/TYPE/GNSS] 
        let (version_str, remainder) = line.split_at(20);
        let (rinex_type, remainder) = remainder.trim().split_at(20);
        let (constellation, remainder) = remainder.trim().split_at(20);
        println!("RINEX | VERSION \"{}\" | TYPE \"{}\" | OTHER \"{}\"", version_str, rinex_type, constellation);
        
        let version = version::Version::from_str(version_str.trim())?;
        if !version.is_supported() {
            return Err(HeaderError::VersionNotSupported(String::from(version_str)))
        }

        // line2
        line = lines.next()
            .unwrap();
        // comments ?
        while is_rinex_comment!(line) {
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
        let run_by: String = match run_by.trim().eq("") {
            true => String::from("Unknown"),
            false => String::from(run_by.trim()) 
        };
        let (date_str, _) = remainder.split_at(20);
        // identify date format (UTC/LCL)
        let date_str: &str = match date_str.contains("UTC") {
            true => {
                let offset = date_str.rfind("UTC")
                    .unwrap();
                date_str.split_at(offset).0.trim() 
            },
            false => {
                match date_str.contains("LCL") {
                    true => {
                        let offset = date_str.rfind("LCL")
                            .unwrap();
                        date_str.split_at(offset).0.trim() 
                    },
                    false => { // some files do not exhibit UTC/LCL marker
                        date_str.trim()
                    }
                }
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
        while is_rinex_comment!(line) {
            line = lines.next()
                .unwrap()
        }

        // order may vary from now on
        // indentifiers
        let mut station    = String::from("Unknown");
        let mut station_id = String::from("Unknown");
        let mut observer   = String::from("Unknwon");
        let mut agency     = String::from("Unknwown");
        // hardware
        let mut ant        : Option<Antenna> = None;
        let mut ant_coords : Option<rust_3d::Point3D> = None;
        let mut rcvr       : Option<Rcvr>    = None;
        // other
        let mut leap       : Option<u32>     = None;
        let mut sampling_interval: Option<f32> = None;
        let mut rcvr_clock_offset_applied: Option<bool> = None;
        let mut coords     : Option<rust_3d::Point3D> = None;
        let mut epochs: (Option<GnssTime>, Option<GnssTime>) = (None, None);
        loop {
            if line.contains("MARKER NAME") {
                station = String::from(line.split_at(20).0.trim())
            } else if line.contains("MARKER NUMBER") {
                station_id = String::from(line.split_at(20).0.trim()) 
            } else if line.contains("OBSERVER / AGENCY") {
                let (obs, ag) = line.split_at(20);
                let ag = ag.split_at(20).0;
                agency = String::from(ag.trim())

            } else if line.contains("REC # / TYPE / VERS") {
                rcvr = Some(Rcvr::from_str(line)?) 
            
            } else if line.contains("ANT # / TYPE") {
                let (id, remainder) = line.split_at(20);
                let (make, remainder) = remainder.trim().split_at(20);
                println!("ANTENNA | ID \"{}\" | MAKE \"{}\"", id, make);
            
            } else if line.contains("LEAP SECOND") {
                // TODO
                // LEAP SECOND might have complex format
                //let leap_str = line.split_at(20).0.trim();
                //leap = Some(u32::from_str_radix(leap_str, 10)?)

            } else if line.contains("DOI") {
                // TODO digital object identifier
                // v> 4.0
            } else if line.contains("MERGED FILE") {
                //TODO nb# of merged files
                // v>4.0
            } else if line.contains("STATION INFORMATION") {
                // TODO station URL
                // v>4
            } else if line.contains("LICENSE OF USE") {
                // TODO license in use
                // v>4
            
            
            } else if line.contains("TIME OF FIRST OBS") {
                let items: Vec<&str> = line.split_ascii_whitespace()
                    .collect();
                let (y, month, d, h, min, s, constel): (i32,u32,u32,u32,u32,f32,constellation::Constellation) =
                    (i32::from_str_radix(items[0].trim(),10)?,
                    u32::from_str_radix(items[1].trim(),10)?,
                    u32::from_str_radix(items[2].trim(),10)?,
                    u32::from_str_radix(items[3].trim(),10)?,
                    u32::from_str_radix(items[4].trim(),10)?,
                    f32::from_str(items[5].trim())?,
                    constellation::Constellation::from_str(items[6].trim())?);
                let utc = chrono::NaiveDate::from_ymd(y,month,d).and_hms(h,min,s as u32);
                epochs.0 = Some(GnssTime::new(utc, constel)) 

            } else if line.contains("TIME OF LAST OBS") {
                let items: Vec<&str> = line.split_ascii_whitespace()
                    .collect();
                let (y, month, d, h, min, s, constel): (i32,u32,u32,u32,u32,f32,constellation::Constellation) =
                    (i32::from_str_radix(items[0].trim(),10)?,
                    u32::from_str_radix(items[1].trim(),10)?,
                    u32::from_str_radix(items[2].trim(),10)?,
                    u32::from_str_radix(items[3].trim(),10)?,
                    u32::from_str_radix(items[4].trim(),10)?,
                    f32::from_str(items[5].trim())?,
                    constellation::Constellation::from_str(items[6].trim())?);
                let utc = chrono::NaiveDate::from_ymd(y,month,d).and_hms(h,min,s as u32);
                epochs.1 = Some(GnssTime::new(utc, constel)) 
            
            } else if line.contains("WAVELENGTH FACT L1/2") {
     //1     1                                                WAVELENGTH FACT L1/2
            
            } else if line.contains("APPROX POSITION XYZ") {
                let items: Vec<&str> = line.split_ascii_whitespace()
                    .collect();
                let (x, y, z): (f64,f64,f64) = 
                    (f64::from_str(items[0].trim())?,
                    f64::from_str(items[1].trim())?,
                    f64::from_str(items[2].trim())?);
                coords = Some(rust_3d::Point3D::new(x,y,z))

            } else if line.contains("ANTENNA: DELTA H/E/N") {
                //TODO
        //0.0000        0.0000        0.0000                  ANTENNA: DELTA H/E/N
            } else if line.contains("ANTENNA: DELTA X/Y/Z") {
                let items: Vec<&str> = line.split_ascii_whitespace()
                    .collect();
                let (x, y, z): (f64,f64,f64) = 
                    (f64::from_str(items[0].trim())?,
                    f64::from_str(items[1].trim())?,
                    f64::from_str(items[2].trim())?);
                ant_coords = Some(rust_3d::Point3D::new(x,y,z))

            } else if line.contains("ANTENNA: B.SIGHT XYZ") {
                //TODO
            } else if line.contains("ANTENNA: ZERODIR XYZ") {
                //TODO
            } else if line.contains("CENTER OF MASS: XYZ") {
                //TODO
            } else if line.contains("ANTENNA: PHASECENTER") {
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
            } else if line.contains("SYS / PVCS APPLIED") {
                // RINEX::ClockData specific 
                // + satellite system (G/R/E/C/I/J/S)
                // + programe name to apply Phase Center Variation
                // + source of corrections (url)
                // <o repeated for each satellite system
                // <o blank field when no corrections applied
            } else if line.contains("# / TYPES OF DATA") {
                // RINEX::ClockData specific 
                // + number of different clock data types stored
                // + list of clock data  types
            } else if line.contains("# / TYPES OF OBSERV") {
     //5    L1    L2    C1    P1    P2                        # / TYPES OF OBSERV
            
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
            } else if line.contains("IONOSPHERIC CORR") {
                // TODO
                // GPSA 0.1025E-07 0.7451E-08 -0.5960E-07 -0.5960E-07
                // GPSB 0.1025E-07 0.7451E-08 -0.5960E-07 -0.5960E-07

            } else if line.contains("TIME SYSTEM CORR") {
                // TODO
                // GPUT 0.2793967723E-08 0.000000000E+00 147456 1395
            
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
            while is_rinex_comment!(line) {
                line = lines.next()
                    .unwrap()
            }
        }
        
        Ok(Header{
            version: version,
            crinex: crinex_infos, 
            rinex_type: RinexType::from_str(rinex_type.trim())?,
            constellation: constellation::Constellation::from_str(constellation.trim())?, 
            program: String::from(pgm.trim()),
            run_by: run_by,
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

    /// Returns `Rinex` Version number
    pub fn get_rinex_version (&self) -> version::Version { self.version }
    /// Returns `Rinex` type 
    pub fn get_rinex_type (&self) -> RinexType { self.rinex_type }
    /// Returns `GNSS` constellation
    pub fn get_constellation (&self) -> constellation::Constellation { self.constellation }
}
