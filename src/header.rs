//! Describes a `RINEX` header, includes
//! rinex header parser and associated methods
//use regex::Regex;
use thiserror::Error;

use crate::version;
use crate::gnss_time;
use crate::{is_comment, Type, TypeError};
use crate::constellation::{Constellation, ConstellationError};
use crate::meteo::{MeteoDataCode, MeteoDataCodeError};
use crate::observation::{ObservationCode, ObservationCodeError};

/// Describes a `CRINEX` (compact rinex) 
pub const CRINEX_MARKER_COMMENT : &str = "COMPACT RINEX FORMAT";
/// End of Header section reached
pub const HEADER_END_MARKER : &str = "END OF HEADER";

/// GNSS receiver description
#[derive(Debug)]
pub struct Rcvr {
    model: String, 
    sn: String, // serial #
    firmware: String, // firmware #
}

impl Default for Rcvr {
    /// Builds a `default` Receiver
    fn default() -> Rcvr {
        Rcvr {
            model: String::from("Unknown"),
            sn: String::from("Unknown"),
            firmware: String::from("Unknown"),
        }
    }
}

impl std::str::FromStr for Rcvr {
    type Err = std::io::Error;
    fn from_str (line: &str) -> Result<Self, Self::Err> {
        let (id, rem) = line.split_at(20);
        let (make, rem) = rem.split_at(20);
        let (version, _) = rem.split_at(20);
        Ok(Rcvr{
            sn: String::from(id.trim()),
            model: String::from(make.trim()),
            firmware: String::from(version.trim()),
        })
    }
}

/// Antenna description 
#[derive(Debug)]
pub struct Antenna {
    /// Hardware model / make descriptor
    model: String,
    /// Serial number / identification number
    sn: String,
    /// 3D coordinates of reference point
    coords: Option<rust_3d::Point3D>,
    /// height in comparison to ref. point
    height: Option<f32>,
    /// eastern eccentricity compared to ref. point
    eastern_eccentricity: Option<f32>,
    /// northern eccentricity compare to ref. point
    northern_eccentricity: Option<f32>,
}

impl Default for Antenna {
    /// Builds default `Antenna` structure
    fn default() -> Antenna {
        Antenna {
            model: String::from("Unknown"),
            sn: String::from("Unknown"),
            coords: None,
            height: None,
            eastern_eccentricity: None,
            northern_eccentricity: None,
        }
    }
}

impl std::str::FromStr for Antenna {
    type Err = std::io::Error;
    fn from_str (line: &str) -> Result<Self, Self::Err> {
        let (id, rem) = line.split_at(20);
        let (make, _) = rem.split_at(20);
        Ok(Antenna{
            sn: String::from(id.trim()),
            model: String::from(make.trim()),
            coords: None,
            height: None,
            eastern_eccentricity: None,
            northern_eccentricity : None,
        })
    }
}

impl Antenna {
    /// Assigns antenna coordinates
    pub fn set_coords (&mut self, coords: rust_3d::Point3D) { self.coords = Some(coords) }
    /// Sets Antenna height compared to reference point
    pub fn set_height (&mut self, height: f32) { self.height = Some(height) }
    /// Sets Eastern eccentricity (m)
    pub fn set_eastern_eccentricity (&mut self, e: f32) { self.eastern_eccentricity = Some(e) }
    /// Sets Northern eccentricity (m)
    pub fn set_northern_eccentricity (&mut self, e: f32) { self.northern_eccentricity = Some(e) }
}

/// `LeapSecond` to describe leap seconds.
/// GLO = UTC = GPS - ΔtLS   
/// GPS = GPS = UTC + ΔtLS   
#[derive(Copy, Clone, Debug)]
pub struct LeapSecond {
    /// current number
    leap: u32,
    /// future or past leap seconds (ΔtLS)   
    past_future: u32,
    /// week number
    week: u32,
    /// day number
    day: u32,
}

impl Default for LeapSecond {
    /// Builds a default (null) `LeapSecond`
    fn default() -> LeapSecond {
        LeapSecond {
            leap: 0,
            past_future: 0,
            week: 0,
            day: 0,
        }
    }
}

impl std::str::FromStr for LeapSecond {
    type Err = Error; 
    /// Builds `LeapSecond` from string descriptor
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ls = LeapSecond::default();
        // leap seconds might have either simple or complex format
        let items: Vec<&str> = s.split_ascii_whitespace()
            .collect();
        match items.len() > 2 {
            false => {
                ls.leap = u32::from_str_radix(items[0].trim(),10)?
            },
            true => {
                let (leap, rem) = s.split_at(6);
                let (past, rem) = rem.split_at(6);
                let (week, rem) = rem.split_at(6);
                let (day, _) = rem.split_at(6);
                ls.leap = u32::from_str_radix(leap.trim(),10)?;
                ls.past_future = u32::from_str_radix(past.trim(),10)?;
                ls.week = u32::from_str_radix(week.trim(),10)?;
                ls.day = u32::from_str_radix(day.trim(),10)?
            },
        }
        Ok(ls)
    }
}

/// Describes `Compact RINEX` specific information
#[derive(Debug)]
pub struct CrinexInfo {
    version: String, // compression version
    prog: String, // compression program
    date: chrono::NaiveDateTime, // date of compression
}

/// Describes known marker types
enum MarkerType {
    /// Earth fixed & high precision
    Geodetic,
    /// Earth fixed & low precision
    NonGeodetic,
    /// Generated from network
    NonPhysical,
    /// Orbiting space vehicule
    Spaceborne,
    /// Aircraft, balloon..
    Airborne,
    /// Mobile water craft
    WaterCraft,
    /// Mobile terrestrial vehicule
    GroundCraft,
    /// Fixed on water surface
    FixedBuoy,
    /// Floating on water surface
    FloatingBuoy,
    /// Floating on ice
    FloatingIce, 
    /// Fixed on glacier
    Glacier,
    /// Rockets, shells, etc..
    Ballistic,
    /// Animal carrying a receiver
    Animal,
    /// Human being carrying a receiver
    Human,
}

/// Describes `RINEX` file header
#[derive(Debug)]
pub struct RinexHeader {
    /// revision for this `RINEX`
    pub version: version::Version, 
    /// optionnal `CRINEX` (compact `RINEX` infos), 
    /// if this is a CRINEX
    pub crinex: Option<CrinexInfo>, 
    /// type of `RINEX` file
    pub rinex_type: Type, 
    /// `GNSS` constellation being used
    pub constellation: Constellation, 
    /// program name
    pub program: String, 
    /// program `run by`
    pub run_by: String, // program run by
    /// station label
    pub station: String, 
    /// station identifier
    pub station_id: String, 
    /// optionnal station URL 
    pub station_url: Option<String>, 
    /// name of observer
    pub observer: String, 
    /// name of production agency
    pub agency: String, 
    /// optionnal hardware (receiver) infos
    pub rcvr: Option<Rcvr>, 
    /// optionnal antenna infos
    pub ant: Option<Antenna>, 
    /// optionnal leap seconds infos
    pub leap: Option<LeapSecond>, 
    /// station approxiamte coordinates
    pub coords: Option<rust_3d::Point3D>, 
    /// optionnal observation wavelengths
    pub wavelengths: Option<(u32,u32)>, 
    /// optionnal sampling interval (s)
    pub sampling_interval: Option<f32>, 
    /// optionnal (first, last) epochs
    pub epochs: (Option<gnss_time::GnssTime>, Option<gnss_time::GnssTime>),
    /// optionnal file license
    pub license: Option<String>,
    /// optionnal Object Identifier (IoT)
    pub doi: Option<String>,
    /// optionnal GPS/UTC time difference
    pub gps_utc_delta: Option<u32>,
    /// processing:   
    /// optionnal data scaling
    pub data_scaling: Option<f64>,
    // optionnal ionospheric compensation param(s)
    //ionospheric_corr: Option<Vec<IonoCorr>>,
    // possible time system correction(s)
    //gnsstime_corr: Option<Vec<gnss_time::GnssTimeCorr>>,
    /// Observations:   
    /// true if epochs & data compensate for local clock drift 
    pub rcvr_clock_offset_applied: Option<bool>, 
    // observation (specific)
    /// lists all types of observations 
    /// contained in this `Rinex` OBS file
    pub obs_codes: Option<Vec<String>>, 
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Compact RINEX related content mismatch")]
    CrinexFormatError,
    #[error("RINEX version is not supported '{0}'")]
    VersionNotSupported(String),
    #[error("Line \"{0}\" should begin with Rinex version \"x.yy\"")]
    VersionFormatError(String),
    #[error("rinex type error")]
    TypeError(#[from] TypeError),
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
    #[error("failed to parse leap second from \"{0}\"")]
    LeapSecondParsingError(String),
    #[error("observation type code not recognized \"{0}\"")]
    ObservationTypeCodeError(#[from] ObservationCodeError),
    #[error("meteo data code not recognized \"{0}\"")]
    MeteoDataCodeError(#[from] MeteoDataCodeError),
}

impl Default for RinexHeader {
    fn default() -> RinexHeader {
        RinexHeader {
            version: version::Version::default(), 
            crinex: None,
            rinex_type: Type::default(),
            constellation: Constellation::default(),
            program: String::from("Unknown"),
            run_by: String::from("Unknown"),
            station: String::from("Unknown"),
            station_id: String::from("Unknown"),
            observer: String::from("Unknown"),
            agency: String::from("Unknown"),
            station_url: None,
            leap: None,
            license: None,
            doi: None,
            gps_utc_delta: None,
            // hardware
            rcvr: None,
            // antenna
            ant: None,
            coords: None, 
            // observations
            epochs: (None, None),
            wavelengths: None,
            obs_codes: None,
            // processing
            rcvr_clock_offset_applied: None,
            data_scaling: None,
            //ionospheric_corr: None,
            //gnsstime_corr: None,
            sampling_interval: None,
        }
    }
}

impl std::str::FromStr for RinexHeader {
    type Err = Error;
    /// Builds header from extracted header description
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        let mut lines = content.lines();
        let mut line = lines.next()
            .unwrap();
        // comments ?
        while is_comment!(line) {
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
        while is_comment!(line) {
            line = lines.next()
                .unwrap()
        }

        // line1 {} {} {} // label [VERSION/TYPE/GNSS] 
        let (version_str, remainder) = line.split_at(20);
        let (type_str, remainder) = remainder.trim().split_at(20);
        let (constellation_str, _) = remainder.trim().split_at(20);

        let rinex_type = Type::from_str(type_str.trim())?;
        let constellation: Constellation;
        
        if type_str.trim().contains("GLONASS") {
            // special case, sometimes GLONASS NAV
            // drops the constellation field cause it's implied
            constellation = Constellation::Glonass
        } else {
            constellation = Constellation::from_str(constellation_str.trim())?
        }

        let version = version::Version::from_str(version_str.trim())?;
        if !version.is_supported() {
            return Err(Error::VersionNotSupported(String::from(version_str)))
        }

        // line2
        line = lines.next()
            .unwrap();
        // comments ?
        while is_comment!(line) {
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
/*
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
        while is_comment!(line) {
            line = lines.next()
                .unwrap()
        }

        // order may vary from now on
        // indentifiers
        let mut station    = String::from("Unknown");
        let mut station_id = String::from("Unknown");
        let mut observer   = String::from("Unknown");
        let mut agency     = String::from("Unknown");
        let mut license     : Option<String> = None;
        let mut doi         : Option<String> = None;
        let mut station_url : Option<String> = None;
        // hardware
        let mut ant        : Option<Antenna> = None;
        let mut ant_coords : Option<rust_3d::Point3D> = None;
        let mut ant_hen    : Option<(f32,f32,f32)> = None;
        let mut rcvr       : Option<Rcvr>    = None;
        // other
        let mut leap       : Option<LeapSecond> = None;
        let mut sampling_interval: Option<f32> = None;
        let mut rcvr_clock_offset_applied: Option<bool> = None;
        let mut coords     : Option<rust_3d::Point3D> = None;
        let mut epochs: (Option<gnss_time::GnssTime>, Option<gnss_time::GnssTime>) = (None, None);
        // (OBS) 
        let obs_nb_sat : u32 = 0;
        let mut obs_codes  : Vec<String> = Vec::new();

        loop {
            /*
            <o
                the "number of satellites" also corresponds
                to the number of records of the same epoch
                following the 'epoch' record.
                If may be used to skip appropriate number of data records if the event flags are not to be evaluated in detail

            */
            if line.contains("MARKER NAME") {
                station = String::from(line.split_at(20).0.trim())
            } else if line.contains("MARKER NUMBER") {
                station_id = String::from(line.split_at(20).0.trim()) 
            } else if line.contains("OBSERVER / AGENCY") {
                let (obs, ag) = line.split_at(20);
                observer = String::from(obs.trim());
                agency = String::from(ag.trim())

            } else if line.contains("REC # / TYPE / VERS") {
                rcvr = Some(Rcvr::from_str(line)?) 
            
            } else if line.contains("ANT # / TYPE") {
                ant = Some(Antenna::from_str(line)?)
            
            } else if line.contains("LEAP SECOND") {
                leap = Some(LeapSecond::from_str(line.split_at(40).0)?)

            } else if line.contains("DOI") {
                let (content, _) = line.split_at(40); //  TODO: confirm please
                doi = Some(String::from(content.trim()))

            } else if line.contains("MERGED FILE") {
                //TODO V > 3 nb# of merged files

            } else if line.contains("STATION INFORMATION") {
                let (url, _) = line.split_at(40); //TODO confirm please 
                station_url = Some(String::from(url.trim()))

            } else if line.contains("LICENSE OF USE") {
                let (lic, _) = line.split_at(40); //TODO confirm please 
                license = Some(String::from(lic.trim()))
            
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
                epochs.0 = Some(gnss_time::GnssTime::new(utc, constel)) 

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
                epochs.1 = Some(gnss_time::GnssTime::new(utc, constel)) 
            
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
                let (h, rem) = line.split_at(15);
                let (e, rem) = rem.split_at(15);
                let (n, _) = rem.split_at(15);
                ant_hen = Some((
                    f32::from_str(h.trim())?,
                    f32::from_str(e.trim())?,
                    f32::from_str(n.trim())?))

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
                // will always appear prior PRN/#OBS
                // determines nb of satellites in observation file
                //let (nb, _) = line.split_at(10);
                //obs_nb_sat = u32::from_str_radix(nb.trim(), 10)?

            } else if line.contains("PRN / # OF OBS") {
                let (sv, _) = line.split_at(7);
                if sv.trim().len() > 0 {
                    
                }
                // lists all Sv
                //let items: Vec<&str> = line.split_ascii_whitespace()
                //    .collect();
                 
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
            } else if line.contains("TYPES OF OBS") { 
                // Rinex V < 3 : old fashion obs data
                // ⚠ ⚠ could either be observation or meteo data
                let (rem, _) = line.split_at(60);
                match rinex_type {
                    Type::ObservationData => {
                        let (_, mut rem) = rem.split_at(6);
                        loop {
                            let (code, r) = rem.split_at(6);
                            if code.trim().len() == 0 {
                                break
                            }
                            obs_codes.push(code.trim().to_string());
                            if r.len() == 0 {
                                break
                            }
                            rem = r
                        }
                    },
                    Type::MeteorologicalData => {
                        let (_, mut rem) = rem.split_at(6);
                        loop {
                            let (code, r) = rem.split_at(6);
                            if code.trim().len() == 0 {
                                break
                            }
                            obs_codes.push(code.trim().to_string());
                            if r.len() == 0 {
                                break
                            }
                            rem = r
                        }
                    },
                    _ => {},
                }
            } else if line.contains("SYS / # / OBS TYPES") {
                //let content = line.split_at(60).0;
/*G   22 C1C L1C D1C S1C C1W S1W C2W L2W D2W S2W C2L L2L D2L  SYS / # / OBS TYPES
       S2L C5Q L5Q D5Q S5Q C1L L1L D1L S1L                  SYS / # / OBS TYPES
E   20 C1C L1C D1C S1C C6C L6C D6C S6C C5Q L5Q D5Q S5Q C7Q  SYS / # / OBS TYPES
       L7Q D7Q S7Q C8Q L8Q D8Q S8Q                          SYS / # / OBS TYPES
S    8 C1C L1C D1C S1C C5I L5I D5I S5I                      SYS / # / OBS TYPES
R   20 C1C L1C D1C S1C C1P L1P D1P S1P C2P L2P D2P S2P C2C  SYS / # / OBS TYPES
       L2C D2C S2C C3Q L3Q D3Q S3Q                          SYS / # / OBS TYPES
C   20 C1P L1P D1P S1P C5P L5P D5P S5P C2I L2I D2I S2I C7I  SYS / # / OBS TYPES
       L7I D7I S7I C6I L6I D6I S6I                          SYS / # / OBS TYPES
I    4 C5A L5A D5A S5A                                      SYS / # / OBS TYPES*/
                // Rinex::ObservationData 
                // types of observation for specified constellation
                
                // grab Constellation info:
                //   [+] if constellation info exists: new entry
                //   [+] otherwise: adding some more to a previous entry
                let (sys, rem) = line.split_at(3);
                let (nb, rem) = rem.split_at(3);
                if sys.trim().len() > 0 {
                    println!("SYS \"{}\" NB \"{}\" REM \"{}\"", sys.trim(), nb.trim(), rem.trim());
                } else {
                    println!("REM \"{}\"", rem.trim());
                }
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
            while is_comment!(line) {
                line = lines.next()
                    .unwrap()
            }
        }
        
        if ant.is_some() { // whatever.. but will avoid hypothetical crash
            if ant_coords.is_some() { // in case of faulty producer
                ant.as_mut().unwrap().set_coords(ant_coords.unwrap())
            }
            if ant_hen.is_some() {
                ant.as_mut().unwrap().set_height(ant_hen.unwrap().0);
                ant.as_mut().unwrap().set_eastern_eccentricity(ant_hen.unwrap().1);
                ant.as_mut().unwrap().set_northern_eccentricity(ant_hen.unwrap().2);
            }
        }

        let obs_codes: Option<Vec<String>> = match obs_codes.len() {
            0 => None,
            _ => Some(obs_codes)
        };
        
        Ok(RinexHeader{
            version: version,
            crinex: crinex_infos, 
            rinex_type,
            constellation,
            program: String::from(pgm.trim()),
            run_by,
            station,
            station_id,
            agency,
            observer,
            license,
            doi,
            station_url,
            rcvr, 
            ant, 
            leap,
            coords: coords,
            // ?
            wavelengths: None,
            gps_utc_delta: None,
            // observations
            epochs: epochs,
            obs_codes,
            // processing
            sampling_interval: sampling_interval,
            data_scaling: None,
            //ionospheric_corr: None,
            //gnsstime_corr: None,
            rcvr_clock_offset_applied,
        })
    }
}

impl RinexHeader {
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
