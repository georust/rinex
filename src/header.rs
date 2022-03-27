//! Describes a `RINEX` header, includes
//! rinex header parser and associated methods
use thiserror::Error;
use std::collections::HashMap;

use crate::version;
use crate::gnss_time;
use crate::{is_comment, Type, TypeError};
use crate::constellation;
use crate::constellation::{Constellation, ConstellationError};

/// Describes a `CRINEX` (compressed rinex) 
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

/// Meteo Observation Sensor
#[derive(Debug)]
pub struct Sensor {
	/// Model of this sensor
	model: String,
	/// Type of sensor
	sens_type: String,
	/// Sensor accuracy [°C,..]
	accuracy: f32,
	/// Physics measured by this sensor
	physics: String,
}

impl Sensor {
	/// Builds a new Meteo Obs sensor,
	/// with given `model`, `sensor type` `accuracy` and `physics` fields
	pub fn new (model: &str, sens_type: &str, accuracy: f32, physics: &str) -> Sensor {
		Sensor {
			model: String::from(model),
			sens_type: String::from(sens_type),
			accuracy,
			physics: String::from(physics),
		}
	}
}

/// Antenna description 
#[derive(Debug)]
pub struct Antenna {
    /// Hardware model / make descriptor
    pub model: String,
    /// Serial number / identification number
    pub sn: String,
    /// 3D coordinates of reference point
    pub coords: Option<rust_3d::Point3D>,
    /// height in comparison to ref. point
    pub height: Option<f32>,
    /// eastern eccentricity compared to ref. point
    pub eastern_eccentricity: Option<f32>,
    /// northern eccentricity compare to ref. point
    pub northern_eccentricity: Option<f32>,
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
    pub fn new (sn: &str, model: &str, 
        coords: Option<rust_3d::Point3D>,
            h : Option<f32>, e: Option<f32>, n: Option<f32>) -> Antenna {
        Antenna {
            sn: sn.to_string(),
            model: model.to_string(),
            coords: coords,
            height: h,
            northern_eccentricity: n,
            eastern_eccentricity: e,
        }
    }
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
#[derive(Clone, Debug)]
pub struct CrinexInfo {
    pub version: version::Version, // compression version
    pub prog: String, // compression program
    pub date: chrono::NaiveDateTime, // date of compression
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
pub struct Header {
    /// revision for this `RINEX`
    pub version: version::Version, 
    /// optionnal `CRINEX` (compressed `RINEX` infos), 
    /// if this is a CRINEX
    pub crinex: Option<CrinexInfo>, 
    /// type of `RINEX` file
    pub rinex_type: Type, 
    /// specific `GNSS` constellation system,
	/// may not exist for RINEX files 
    pub constellation: Option<Constellation>, 
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
    /// optionnal comments encountered in header section, 
    /// exposed as is
    pub comments : Vec<String>,
    /// optionnal hardware (receiver) infos
    pub rcvr: Option<Rcvr>, 
    /// optionnal antenna infos
    pub ant: Option<Antenna>, 
	/// optionnal meteo sensors infos
	pub sensors: Option<Vec<Sensor>>,
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
    pub rcvr_clock_offset_applied: bool, 
    // observation (specific)
    /// lists all types of observations 
    /// contained in this `Rinex` OBS file
    pub obs_codes: Option<HashMap<Constellation, Vec<String>>>, 
	/// lists all types of observations
	/// contains in this `RINEX` Meteo file
    pub met_codes: Option<Vec<String>>, 
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("CRINEX related content mismatch")]
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
}

impl Default for Header {
    fn default() -> Header {
        Header {
            version: version::Version::default(), 
            crinex: None,
            rinex_type: Type::default(),
            constellation: Some(Constellation::default()),
            program: String::from("Unknown"),
            run_by: String::from("Unknown"),
            station: String::from("Unknown"),
            station_id: String::from("Unknown"),
            observer: String::from("Unknown"),
            agency: String::from("Unknown"),
            station_url: None,
            comments: Vec::with_capacity(4),
            leap: None,
            license: None,
            doi: None,
            gps_utc_delta: None,
            // hardware
            rcvr: None,
            ant: None,
			sensors: None,
            coords: None, 
            // observations
            epochs: (None, None),
            wavelengths: None,
            obs_codes: None,
			met_codes: None,
            // processing
            rcvr_clock_offset_applied: false,
            data_scaling: None,
            //ionospheric_corr: None,
            //gnsstime_corr: None,
            sampling_interval: None,
        }
    }
}

impl std::str::FromStr for Header {
    type Err = Error;
    /// Builds header from extracted header description
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        let mut comments : Vec<String> = Vec::with_capacity(4);
        let mut lines = content.lines();
        let mut line = lines.next()
            .unwrap();
        // comments ?
        while is_comment!(line) {
            comments.push(String::from(line.split_at(60).0));
            line = lines.next()
                .unwrap()
        }
        // 'compressed rinex'?
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
                    version: version::Version::from_str(version.trim())?,
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
            comments.push(String::from(line.split_at(60).0));
            line = lines.next()
                .unwrap()
        }

        // line1 {} {} {} // label [VERSION/TYPE/GNSS] 
        let (version_str, remainder) = line.split_at(20);
        let (type_str, remainder) = remainder.trim().split_at(20);
        let (constellation_str, _) = remainder.trim().split_at(20);

        let rinex_type = Type::from_str(type_str.trim())?;
        let constellation: Option<Constellation>;
        
        if type_str.contains("GLONASS") {
            // special case, sometimes GLONASS NAV
            // drops the constellation field cause it's implied
            constellation = Some(Constellation::Glonass)
        } else if type_str.contains("METEOROLOGICAL DATA") {
			// these files are not tied to a constellation system,
			// therefore, do not have this field
			constellation = None
		} else { // regular files
            constellation = Some(Constellation::from_str(constellation_str.trim())?)
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
            comments.push(String::from(line.split_at(60).0));
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
            comments.push(String::from(line.split_at(60).0));
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
		let mut sensors    : Vec<Sensor> = Vec::with_capacity(3);
        // other
        let mut leap       : Option<LeapSecond> = None;
        let mut sampling_interval: Option<f32> = None;
        let mut rcvr_clock_offset_applied: bool = false;
        let mut coords     : Option<rust_3d::Point3D> = None;
        let mut epochs: (Option<gnss_time::GnssTime>, Option<gnss_time::GnssTime>) = (None, None);
        // (OBS) 
        let mut obs_codes  : HashMap<Constellation, Vec<String>> 
            = HashMap::with_capacity(constellation::CONSTELLATION_LENGTH);
		let mut met_codes  : Vec<String> = Vec::with_capacity(3);

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
                let (content, _) = line.split_at(60);
                let (obs, ag) = content.split_at(20);
                observer = String::from(obs.trim());
                agency = String::from(ag.trim())

            } else if line.contains("REC # / TYPE / VERS") {
                rcvr = Some(Rcvr::from_str(line)?) 

			} else if line.contains("SENSOR MOD/TYPE/ACC") {
				let (content, _) = line.split_at(60);
				let (model, rem) = content.split_at(20);
				let (stype, rem) = rem.split_at(20+6);
				let (accuracy, rem) = rem.split_at(7+4);
				//println!("model \"{}\" stype \"{}\" accuracy \"{}\"", model, stype, accuracy);
				let accuracy = f32::from_str(accuracy.trim())?;
				let (physics, _) = rem.split_at(2);
				sensors.push(Sensor::new(model.trim(),stype.trim(),accuracy,physics.trim()));
				//println!("sensor {:#?}", sensors)
            
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
                /*let items: Vec<&str> = line.split_ascii_whitespace()
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
                epochs.0 = Some(gnss_time::GnssTime::new(utc, constel)) */

            } else if line.contains("TIME OF LAST OBS") {
               /* let items: Vec<&str> = line.split_ascii_whitespace()
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
                epochs.1 = Some(gnss_time::GnssTime::new(utc, constel))*/ 
            
            } else if line.contains("WAVELENGTH FACT L1/2") {
                //TODO

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
                rcvr_clock_offset_applied = i32::from_str_radix(ok_str, 10)? != 0

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
				let (n_codes, mut line) = rem.split_at(6);
				let n_codes = u8::from_str_radix(n_codes.trim(), 10)?;
				let n_lines : usize = num_integer::div_ceil(n_codes, 9).into();
                let mut codes : Vec<String> = Vec::new();

				for i in 0..n_lines {
					let content : Vec<&str> = line.split_ascii_whitespace()
						.collect();
					for j in 0..content.len() { 
                    	codes.push(String::from(content[j].trim()));
					}
					if i < n_lines-1 { // takes more than one line
						line = lines.next() // --> need to grab new content
							.unwrap();
						line = line.split_at(60).0 // remove comments
					}
				}

                // build code map 
                // to be used later on when parsing payload
                match constellation {
                    Some(Constellation::Mixed) => {
						// Multi constell:
						//  trick to later identify, in all cases
                        let constells : Vec<Constellation> =
						Vec::from([
                            Constellation::GPS,
                            Constellation::Glonass,
                            Constellation::Galileo,
                            Constellation::Beidou,
                            Constellation::Sbas,
                            Constellation::QZSS,
                        ]);
                		for i in 0..constells.len() {
                    		obs_codes.insert(constells[i], codes.clone());
                		}
                    },
                    Some(c) => {
						// Single constellation system
						obs_codes.insert(c, codes.clone());
					},
					_ => {
						// this is a meteo file,
						// meteo observations are not tied to a specific
						// constellation system
						for i in 0..codes.len() {
							met_codes.push(codes[i].clone())
						}
					},
                };

            } else if line.contains("SYS / # / OBS TYPES") {
                // modern obs code descriptor 
                let (line, _) = line.split_at(60); // remove header
                let (identifier, rem) = line.split_at(1);
                let (n_codes, mut line) = rem.split_at(5);
                let n_codes = u8::from_str_radix(n_codes.trim(), 10)?;
				let n_lines : usize = num_integer::div_ceil(n_codes, 13).into(); 
                let mut codes : Vec<String> = Vec::with_capacity(n_codes.into());
                let constell : constellation::Constellation = match identifier {
                    "G" => constellation::Constellation::GPS,
                    "R" => constellation::Constellation::Glonass,
                    "J" => constellation::Constellation::QZSS,
                    "E" => constellation::Constellation::Galileo,
                    "C" => constellation::Constellation::Beidou,
                    "S" => constellation::Constellation::Sbas,
                    _ => continue, // should never happen 
                };

                for i in 0..n_lines {
					let content : Vec<&str> = line.split_ascii_whitespace()
						.collect();
					for j in 0..content.len() { 
                    	codes.push(String::from(content[j].trim()));
					}
					if i < n_lines-1 { // takes more than one line
						line = lines.next() // --> need to grab new content
							.unwrap();
						line = line.split_at(60).0 // remove comments
					}
                }
                obs_codes.insert(constell, codes);
            
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
                comments.push(String::from(line.split_at(60).0));
                line = lines.next()
                    .unwrap()
            }
        }
        
        let ant : Option<Antenna> = match ant {
            Some(antenna) => {
                Some(Antenna::new(
                    &antenna.sn, 
                    &antenna.model, 
                    ant_coords, 
                    Some(ant_hen.unwrap_or((0.0_f32,0.0_f32,0.0_f32)).0), 
                    Some(ant_hen.unwrap_or((0.0_f32,0.0_f32,0.0_f32)).1), 
                    Some(ant_hen.unwrap_or((0.0_f32,0.0_f32,0.0_f32)).2), 
                ))
            },
            _ => None,
        };

		let sensors : Option<Vec<Sensor>> = match sensors.len() > 0 {
			true => {
				// identified some sensors
				Some(sensors)
			},
			false => None
		};

        let obs_codes: Option<HashMap<Constellation, Vec<String>>> = match obs_codes.is_empty() {
            true => None,
            false => Some(obs_codes),
        };

		let met_codes : Option<Vec<String>> = match met_codes.is_empty() {
			true => None,
			false => Some(met_codes),
		};
        
        Ok(Header{
            version: version,
            crinex: crinex_infos, 
            rinex_type,
            constellation,
            comments,
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
			sensors,
            leap,
            coords: coords,
            wavelengths: None,
            gps_utc_delta: None,
            epochs,
            obs_codes,
			met_codes,
            sampling_interval: sampling_interval,
            data_scaling: None,
            //ionospheric_corr: None,
            //gnsstime_corr: None,
            rcvr_clock_offset_applied,
        })
    }
}

impl std::fmt::Display for Header {
    /// `header` formatter, mainly for 
    /// `RINEX` file production purposes
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        /*if self.is_crinex() {
            write!(f,"F9.2{}           ", self.version)?;
            write!(f,"F9.2{}           ", self.version)?;
        }*/
        /*write!(f,"F9.2{}           ", self.version)?;*/
        match self.rinex_type {
            Type::ObservationData => {
                write!(f,"OBSERVATION DATA    ")?
            },
            Type::MeteorologicalData => {
                write!(f,"METEOROLOGICAL DATA ")?
            },
            Type::NavigationMessage => {
                match self.constellation {
                    Some(Constellation::Glonass) => write!(f,"GLONASS NAV        ")?,
                    _ => write!(f,"NAVIGATION DATA    ")?
                }
            },
            _ => {
                panic!("non supported rinex type")
            }
        }
        // PGM / RUN BY / DATE
        write!(f, "{:<20}", self.program)?;
        write!(f, "{:<20}", self.run_by)?; // TODO date field parsing
        write!(f, "{:<20}", "PGM / RUN BY / DATE\n")?; 
        // OBSERVER / AGENCY
        write!(f, "{:<20}", self.observer)?;
        write!(f, "{:<40}", self.agency)?;
        write!(f, "{:<18}", "OBSERVER / AGENCY\n")?; 
        // COMMENTS 
        for comment in self.comments.iter() {
            write!(f, "{:<60}", comment)?;
            write!(f, "COMMENT\n")?
        }
        // END OF HEADER
        write!(f, "{:>73}", "END OF HEADER")
    }
}

impl Header {
    /// Returns true if self is a `Compressed RINEX`
    pub fn is_crinex (&self) -> bool { self.crinex.is_some() }
}
