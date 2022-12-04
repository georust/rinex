//! Describes a `RINEX` header, includes
//! rinex header parser and associated methods
use super::*;
use crate::{
    hardware::{
        Rcvr,
        Antenna,
        SvAntenna,
    },
    reader::BufferedReader,
    types::{
        Type, TypeError,
    },
    meteo,
    ionex,
    clocks,
    antex,
    leap,
    observation,
    observation::Crinex,
    version::Version,
};

use rust_3d::Point3D;
use thiserror::Error;
use std::str::FromStr;
use std::io::prelude::*;
use strum_macros::EnumString;

macro_rules! from_b_fmt_month {
    ($m: expr) => {
        match $m {
            "Jan" => 1,
            "Feb" => 2,
            "Mar" => 3,
            "Apr" => 4,
            "May" => 5,
            "Jun" => 6,
            "Jul" => 7,
            "Aug" => 8,
            "Sep" => 9,
            "Oct" =>10,
            "Nov" =>11,
            "Dec" =>12,
            _ => 1,
        }
    }
}

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "serde")]
use crate::formatter::opt_point3d;

#[derive(Clone, Debug)]
#[derive(PartialEq, Eq)]
#[derive(EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MarkerType {
    /// Earth fixed & high precision
    #[strum(serialize = "GEODETIC", serialize = "Geodetic")]
    Geodetic,
    /// Earth fixed & low precision
    #[strum(serialize = "NON GEODETIC", serialize = "NonGeodetic")]
    NonGeodetic,
    /// Generated from network
    #[strum(serialize = "NON PHYSICAL", serialize = "NonPhysical")]
    NonPhysical,
    /// Orbiting space vehicule
    #[strum(serialize = "SPACE BORNE", serialize = "Spaceborne")]
    Spaceborne,
    /// Aircraft, balloon..
    #[strum(serialize = "AIR BORNE", serialize = "Airborne")]
    Airborne,
    /// Mobile water craft
    #[strum(serialize = "WATER CRAFT", serialize = "Watercraft")]
    Watercraft,
    /// Mobile terrestrial vehicule
    #[strum(serialize = "GROUND CRAFT", serialize = "Groundcraft")]
    Groundcraft,
    /// Fixed on water surface
    #[strum(serialize = "FIXED BUOY", serialize = "FixedBuoy")]
    FixedBuoy,
    /// Floating on water surface
    #[strum(serialize = "FLOATING BUOY", serialize = "FloatingBuoy")]
    FloatingBuoy,
    /// Floating on ice
    #[strum(serialize = "FLOATING ICE", serialize = "FloatingIce")]
    FloatingIce, 
    /// Fixed on glacier
    #[strum(serialize = "GLACIER", serialize = "Glacier")]
    Glacier,
    /// Rockets, shells, etc..
    #[strum(serialize = "BALLISTIC", serialize = "Ballistic")]
    Ballistic,
    /// Animal carrying a receiver
    #[strum(serialize = "ANIMAL", serialize = "Animal")]
    Animal,
    /// Human being carrying a receiver
    #[strum(serialize = "HUMAN", serialize = "Human")]
    Human,
}

impl Default for MarkerType {
    fn default() -> Self { 
        Self::Geodetic 
    }
}

/// Describes `RINEX` file header
#[derive(Clone, Debug)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Header {
    /// revision for this `RINEX`
    pub version: Version, 
    /// type of `RINEX` file
    pub rinex_type: Type, 
    /// `GNSS` constellation system encountered in this file,
    /// or reference GNSS constellation for the following data.
    pub constellation: Option<Constellation>, 
    /// comments extracted from `header` section
    pub comments : Vec<String>,
    /// program name
    pub program: String, 
    /// program `run by`
    pub run_by: String,
    /// program's `date`
    pub date: String, 
    /// station label
    pub station: String, 
    /// station identifier
    pub station_id: String, 
    /// optionnal station URL 
    pub station_url: String, 
    /// name of observer
    pub observer: String, 
    /// name of production agency
    pub agency: String, 
    /// optionnal receiver placement infos
    pub marker_type: Option<MarkerType>,
    /// optionnal leap seconds infos
    pub leap: Option<leap::Leap>, 
    /// Station approximate coordinates
    #[cfg_attr(feature = "serde", serde(default, with = "opt_point3d"))]
    pub coords: Option<Point3D>, 
    /// Optionnal observation wavelengths
    pub wavelengths: Option<(u32,u32)>, 
    /// Optionnal sampling interval (s)
    pub sampling_interval: Option<Duration>, 
    /// Optionnal file license
    pub license: Option<String>,
    /// Optionnal Object Identifier (IoT)
    pub doi: Option<String>,
    /// Optionnal GPS/UTC time difference
    pub gps_utc_delta: Option<u32>,
    /// Optionnal data scaling
    pub data_scaling: Option<f64>,
    /// Optionnal Receiver information
    #[cfg_attr(feature = "serde", serde(default))]
    pub rcvr: Option<Rcvr>, 
    /// Optionnal Receiver Antenna information
    #[cfg_attr(feature = "serde", serde(default))]
    pub rcvr_antenna: Option<Antenna>, 
    /// Optionnal Vehicule Antenna information,
    /// attached to a specifid Sv, only exists in ANTEX records
    #[cfg_attr(feature = "serde", serde(default))]
    pub sv_antenna: Option<SvAntenna>, 
    /// Observation record specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub obs: Option<observation::HeaderFields>,
    /// Meteo record specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub meteo: Option<meteo::HeaderFields>,
    /// Clocks record specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub clocks: Option<clocks::HeaderFields>,
    /// ANTEX record specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub antex: Option<antex::HeaderFields>,
    /// IONEX record specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub ionex: Option<ionex::HeaderFields>,
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
    #[error("constellation error")]
    ConstellationError(#[from] constellation::Error),
    #[error("failed to parse leap from \"{0}\"")]
    LeapParsingError(#[from] leap::Error),
    #[error("failed to parse antenna / receiver infos")]
    AntennaRcvrError(#[from] std::io::Error),
    #[error("failed to parse integer value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse ANTEX fields")]
    AntexParsingError(#[from] antex::record::Error),
    #[error("failed to parse PCV field")]
    ParsePcvError(#[from] antex::pcv::Error),
    #[error("unknown ionex reference")]
    UnknownReferenceIonex(#[from] ionex::system::Error),
    #[error("faulty ionex grid definition")]
    IonexGridError(#[from] ionex::grid::Error),
}

impl Default for Header {
    fn default() -> Header {
        Header {
            version: Version::default(), 
            rinex_type: Type::default(),
            constellation: Some(Constellation::default()),
            comments: Vec::new(),
            program: "rust-rinex".to_string(), 
            run_by: String::new(),
            date: String::new(),
            station: String::new(),
            station_id: String::new(),
            observer: String::new(),
            agency: String::new(),
            marker_type: None,
            station_url: String::new(),
            doi: None, 
            license: None, 
            leap: None,
            gps_utc_delta: None,
            // hardware
            rcvr: None,
            rcvr_antenna: None,
            sv_antenna: None,
            coords: None, 
            wavelengths: None,
            data_scaling: None,
            sampling_interval: None,
            // rinex specific
            obs: None,
			meteo: None,
            clocks: None,
            antex: None,
            ionex: None,
        }
    }
}

impl Header {
    /// Builds a `Header` from stream reader 
    pub fn new (reader: &mut BufferedReader) -> Result<Header, Error> { 
        let mut rinex_type = Type::default();
        let mut constellation : Option<Constellation> = None;
        let mut version = Version::default();
        let mut comments: Vec<String> = Vec::new();
        let mut program = String::new();
        let mut run_by = String::new();
        let mut date = String::new();
        let mut station = String::new();
        let mut station_id = String::new();
        let mut observer = String::new();
        let mut agency = String::new();
        let mut license : Option<String> = None;
        let mut doi : Option<String> = None;
        let mut station_url= String::new();
        let mut marker_type : Option<MarkerType> = None; 
        let mut rcvr : Option<Rcvr> = None;
        let mut rcvr_antenna: Option<Antenna> = None;
        let mut sv_antenna: Option<SvAntenna> = None;
        let mut leap: Option<leap::Leap> = None;
        let mut sampling_interval: Option<Duration> = None;
        let mut coords: Option<Point3D> = None;
        // RINEX specific fields 
        let mut obs_code_lines : u8 = 0; 
        let mut current_code_syst = Constellation::default(); 
        let mut observation = observation::HeaderFields::default();
        let mut meteo = meteo::HeaderFields::default();
        let mut clocks = clocks::HeaderFields::default();
        let mut antex = antex::HeaderFields::default();
        let mut ionex = ionex::HeaderFields::default();
        
        // iterate on a line basis
        let lines = reader.lines();
        for l in lines { 
            let line = l.unwrap();
            if line.len() < 60 {
                continue // --> invalid header content
            }
            let (content, marker) = line.split_at(60);
            ///////////////////////////////
            // [0] END OF HEADER  
            //     --> done parsing
            ///////////////////////////////
            if marker.trim().eq("END OF HEADER") {
                break
            ///////////////////////////////
            // [0*] COMMENTS
            ///////////////////////////////
            } if marker.trim().eq("COMMENT") {
                // --> storing might be useful
                comments.push(content.trim().to_string());
                continue
            
            //////////////////////////////////////
            // [1] CRINEX Special fields
            /////////////////////////////////////
            } else if marker.contains("CRINEX VERS") {
                let version = content.split_at(20).0;
                observation.crinex = Some(Crinex::default()
                    .with_version(Version::from_str(version.trim())?));

            } else if marker.contains("CRINEX PROG / DATE") {
                let (prog, remainder) = content.split_at(20);
                let (_, remainder) = remainder.split_at(20);
                let date = remainder.split_at(20).0.trim();
                let items: Vec<&str> = date.split_ascii_whitespace()
                    .collect();
                if items.len() == 2 {
                    let date: Vec<&str> = items[0].split("-").collect();
                    let time: Vec<&str> = items[1].split(":").collect();
                    if let Ok(d) = u8::from_str_radix(date[0], 10) {
                        let month = from_b_fmt_month!(date[1]);
                        if let Ok(mut y) = i32::from_str_radix(date[2].trim(), 10) {
                            if let Ok(h) = u8::from_str_radix(time[0].trim(), 10) {
                                if let Ok(m) = u8::from_str_radix(time[1].trim(), 10) {
                                    if let Some(crinex) = &mut observation.crinex {
                                        y += 2000; 
                                        let date = Epoch::from_gregorian_utc(y, month, d, h, m, 0, 0);
                                        *crinex = crinex
                                            .with_prog(prog.trim())
                                            .with_date(date);
                                    }
                                }
                            }
                        }
                    }
                }
             
            ////////////////////////////////////////
            // [2] ANTEX special header
            ////////////////////////////////////////
            } else if marker.contains("ANTEX VERSION / SYST") {
                let (vers, system) = content.split_at(8);
                version = Version::from_str(vers.trim())?;
                if let Ok(constell) = Constellation::from_str(system.trim()) {
                    constellation = Some(constell)
                }
                rinex_type = Type::AntennaData;
            
            } else if marker.contains("PCV TYPE / REFANT") {
                let (pcv_str, rem) = content.split_at(20);
                let (rel_type, rem) = rem.split_at(20);
                let (ref_sn, _) = rem.split_at(20);
                if let Ok(mut pcv) = antex::Pcv::from_str(pcv_str.trim()) {
                    if pcv.is_relative() {
                        // try to parse "Relative Type"
                        if rel_type.trim().len() > 0 {
                            pcv = pcv.with_relative_type(rel_type.trim());
                        }
                    }
                    antex = antex.with_pcv(pcv);
                }
                if ref_sn.trim().len() > 0 {
                    antex = antex.with_serial_number(ref_sn.trim())
                }

            } else if marker.contains("TYPE / SERIAL NO") {
                let items: Vec<&str> = content.split_ascii_whitespace()
                    .collect();
                if items.len() == 2 {
                    // Receiver antenna information
                    // like standard RINEX
                    let (model, rem) = content.split_at(20);
                    let (sn, _) = rem.split_at(20);
                    if let Some(a) = &mut rcvr_antenna {
                        *a = a.with_model(model.trim())
                            .with_serial_number(sn.trim());
                    } else {
                        rcvr_antenna = Some(Antenna::default()
                            .with_model(model.trim())
                            .with_serial_number(sn.trim()));
                    }

                } else if items.len() == 4 {
                    // Space Vehicle antenna information
                    // ANTEX RINEX specific
                    let (model, rem) = content.split_at(10);
                    let (svnn, rem) = rem.split_at(10);
                    let (cospar, _) = rem.split_at(10);
                    if let Ok(sv) = Sv::from_str(svnn.trim()) {
                        if let Some(a) = &mut sv_antenna {
                            *a = a
                                .with_sv(sv)
                                .with_model(model.trim())
                                .with_cospar(cospar.trim());
                        } else {
                            sv_antenna = Some(SvAntenna::default()
                                .with_sv(sv)
                                .with_model(model.trim())
                                .with_cospar(cospar.trim()));
                        }
                    }
                }
            
            //////////////////////////////////////
            // [2] IONEX special header 
            //////////////////////////////////////
            } else if marker.contains("IONEX VERSION / TYPE") {
                let (vers_str, rem) = line.split_at(20);
                let (type_str, rem) = rem.split_at(20); 
                let (system_str, _) = rem.split_at(20);
                version = Version::from_str(vers_str.trim())?;
                rinex_type = Type::from_str(type_str.trim())?;
                let ref_system = ionex::RefSystem::from_str(system_str.trim())?;
                ionex = ionex.with_reference_system(ref_system);

            ///////////////////////////////////////
            // ==> from now on
            // RINEX standard / shared attributes
            ///////////////////////////////////////
            } else if marker.contains("RINEX VERSION / TYPE") {
                let (vers, rem) = line.split_at(20);
                let (type_str, rem) = rem.split_at(20); 
                let (constell_str, _) = rem.split_at(20);
                rinex_type = Type::from_str(type_str.trim())?;
                if type_str.contains("GLONASS") {
                    // old GLONASS NAV : no constellation field
                    constellation = Some(Constellation::Glonass)
                } else if type_str.contains("GPS NAV DATA") {
                    // old GPS NAV: no constellation field
                    constellation = Some(Constellation::GPS)

                } else if type_str.contains("METEOROLOGICAL DATA") {
                    // these files are not tied to a constellation system,
                    // therefore, do not have this field
                    constellation = None
                } else { // regular files
                    if let Ok(constell) = Constellation::from_str(constell_str.trim()) {
                        constellation = Some(constell)
                    }
                }
                version = Version::from_str(vers.trim())?;
                if !version.is_supported() {
                    return Err(Error::VersionNotSupported(vers.to_string()));
                }
            
            } else if marker.contains("PGM / RUN BY / DATE") {
                let (pgm, rem) = line.split_at(20);
                program = pgm.trim().to_string();
                let (rb, rem) = rem.split_at(20);
                run_by = match rb.trim().eq("") {
                    true => String::from("Unknown"),
                    false => rb.trim().to_string(), 
                };
                let (date_str, _) = rem.split_at(20);
                date = date_str.trim().to_string()
            
            } else if marker.contains("MARKER NAME") {
                station = content.split_at(20).0.trim().to_string()
            
            } else if marker.contains("MARKER NUMBER") {
                station_id = content.split_at(20).0.trim().to_string()
            
            } else if marker.contains("MARKER TYPE") {
                let code = content.split_at(20).0.trim();
                if let Ok(marker) = MarkerType::from_str(code) {
                    marker_type = Some(marker)
                }
            
            } else if marker.contains("OBSERVER / AGENCY") {
                let (obs, ag) = content.split_at(20);
                observer = obs.trim().to_string();
                agency = ag.trim().to_string()

            } else if marker.contains("REC # / TYPE / VERS") {
                if let Ok(receiver) = Rcvr::from_str(content) {
                    rcvr = Some(receiver)
                }

            } else if marker.contains("SYS / DCBS APPLIED") {
                let (system, rem) = content.split_at(2);
                let (_program, _url) = rem.split_at(18);
                if let Ok(gnss) = Constellation::from_str(system.trim()) {
                    observation
                        .with_dcb_compensation(gnss);
                }

            } else if marker.contains("SYS / SCALE FACTOR") {
                /*let (system, rem) = content.split_at(2);
                let (factor, rem) = rem.split_at(5);*/

			} else if marker.contains("SENSOR MOD/TYPE/ACC") {
                if let Ok(sensor) = meteo::sensor::Sensor::from_str(content) {
                    meteo.sensors.push(sensor)
                }
            
            } else if marker.contains("SENSOR POS XYZ/H") {
                let (x_str, rem) = content.split_at(14);
                let (y_str, rem) = rem.split_at(14);
                let (z_str, rem) = rem.split_at(14);
                let (h_str, phys_str) = rem.split_at(14);
                if let Ok(observable) = meteo::observable::Observable::from_str(phys_str.trim()) {
                    for sensor in meteo.sensors.iter_mut() {
                        if sensor.observable == observable {
                            if let Ok(x) = f64::from_str(x_str.trim()) {
                                if let Ok(y) = f64::from_str(y_str.trim()) {
                                    if let Ok(z) = f64::from_str(z_str.trim()) {
                                        if let Ok(h) = f64::from_str(h_str.trim()) {
                                            *sensor = sensor.
                                                with_position((x,y,z,h))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            
            } else if marker.contains("LEAP SECOND") {
                let leap_str = content.split_at(40).0.trim();
                if let Ok(lleap) = leap::Leap::from_str(leap_str) {
                    leap = Some(lleap)
                }

            } else if marker.contains("DOI") {
                let (content, _) = content.split_at(40); //  TODO: confirm please
                doi = Some(content.trim().to_string())

            } else if marker.contains("MERGED FILE") {
                //TODO V > 3 
                // nb# of merged files

            } else if marker.contains("STATION INFORMATION") {
                let url = content.split_at(40).0; //TODO confirm please 
                station_url = url.trim().to_string()

            } else if marker.contains("LICENSE OF USE") {
                let lic = content.split_at(40).0; //TODO confirm please 
                license = Some(lic.trim().to_string())
            
            } else if marker.contains("WAVELENGTH FACT L1/2") {
                //TODO

            } else if marker.contains("APPROX POSITION XYZ") {
                // station base coordinates
                let items: Vec<&str> = content.split_ascii_whitespace()
                    .collect();
                if let Ok(x) = f64::from_str(items[0].trim()) {
                    if let Ok(y) = f64::from_str(items[1].trim()) {
                        if let Ok(z) = f64::from_str(items[2].trim()) {
                            if let Some(c) = &mut coords {
                                *c = Point3D::new(x, y, z);
                            } else {
                                coords = Some(Point3D::new(x, y, z));
                            }
                        }
                    }
                }

            } else if marker.contains("ANT # / TYPE") {
                let (model, rem) = content.split_at(20);
                let (sn, _) = rem.split_at(20);
                if let Some(a) = &mut rcvr_antenna {
                    *a = a.with_model(model.trim())
                        .with_serial_number(sn.trim());
                } else {
                    rcvr_antenna = Some(Antenna::default()
                        .with_model(model.trim())
                        .with_serial_number(sn.trim()));
                }
            
            } else if marker.contains("ANTENNA: DELTA X/Y/Z") {
                // Antenna Base/Reference Coordinates
                let items: Vec<&str> = content.split_ascii_whitespace()
                    .collect();
                if let Ok(x) = f64::from_str(items[0].trim()) {
                    if let Ok(y) = f64::from_str(items[1].trim()) {
                        if let Ok(z) = f64::from_str(items[2].trim()) {
                            if let Some(a) = &mut rcvr_antenna {
                                *a = a.with_base_coordinates(x, y, z);
                            } else {
                                rcvr_antenna = Some(Antenna::default()
                                    .with_base_coordinates(x, y, z));
                            }
                        }
                    }
                }

            } else if marker.contains("ANTENNA: DELTA H/E/N") {
                // Antenna H/E/N eccentricity components 
                let (h, rem) = content.split_at(15);
                let (e, rem) = rem.split_at(15);
                let (n, _) = rem.split_at(15);
                if let Ok(h) = f64::from_str(h.trim()) {
                    if let Ok(e) = f64::from_str(e.trim()) {
                        if let Ok(n) = f64::from_str(n.trim()) {
                            if let Some(a) = &mut rcvr_antenna {
                                *a = a
                                    .with_height(h)
                                    .with_eastern_component(e)
                                    .with_northern_component(n);
                            } else {
                                rcvr_antenna = Some(Antenna::default()
                                    .with_height(h)
                                    .with_eastern_component(e)
                                    .with_northern_component(n));
                            }
                        }
                    }
                }

            } else if marker.contains("ANTENNA: B.SIGHT XYZ") {
                //TODO
            } else if marker.contains("ANTENNA: ZERODIR XYZ") {
                //TODO
            } else if marker.contains("CENTER OF MASS: XYZ") {
                //TODO
            } else if marker.contains("ANTENNA: PHASECENTER") {
                //TODO
            
            } else if marker.contains("RCV CLOCK OFFS APPL") {
                let value = content.split_at(20).0.trim();
                if let Ok(n) = i32::from_str_radix(value, 10) {
                    observation.clock_offset_applied = n > 0;
                }

            } else if marker.contains("# OF SATELLITES") {
                // ---> we don't need this info,
                //     user can determine it by analyzing the record

            } else if marker.contains("PRN / # OF OBS") {
                // ---> we don't need this info,
                //     user can determine it by analyzing the record
                
            } else if marker.contains("SYS / PHASE SHIFT") {
                //TODO

            } else if marker.contains("SYS / PVCS APPLIED") {
                // RINEX::ClockData specific 
                // + satellite system (G/R/E/C/I/J/S)
                // + programe name to apply Phase Center Variation
                // + source of corrections (url)
                // <o repeated for each satellite system
                // <o blank field when no corrections applied
            
            } else if marker.contains("TYPES OF OBS") { 
                // --> parsing Observables (V<3 old fashion)
                // ⚠ ⚠ could either be observation or meteo data
                if obs_code_lines == 0 { // first line ever
                    let (n, rem) = content.split_at(6);
                    if let Ok(n) = u8::from_str_radix(n.trim(), 10) {
                        obs_code_lines = num_integer::div_ceil(n, 9); // max. items per line

                    } else {
                        continue // failed to identify # of observables
                            // --> we'll continue grabing some header infos
                            //     record builder will not produce much
                    }
                    
                    let codes : Vec<String> = rem
                        .split_ascii_whitespace()
                        .map(|r| r.trim().to_string())
                        .collect();
                    if rinex_type == Type::ObservationData {
                        match constellation {
                            Some(Constellation::Mixed) => {
                                // Old RINEX + Mixed Constellation:
                                // description is not accurate enough to determine which
                                // code will be measured for which constellation
                                // ---> copy them for all major constellations...
                                //      record builder will use the ones it needs
                                let constells : Vec<Constellation> = vec![
                                    Constellation::GPS,
                                    Constellation::Glonass,
                                    Constellation::Galileo,
                                    Constellation::BeiDou,
                                    Constellation::Geo,
                                    Constellation::QZSS,
                                ];
                                for i in 0..constells.len() {
                                    observation.codes.insert(constells[i], codes.clone());
                                } 
                            },
                            Some(constellation) => {
                                observation.codes.insert(constellation, codes.clone());
                            },
                            None => {
                                unreachable!("observation rinex without any constellation specified");
                            }
                        }
                    } else if rinex_type == Type::MeteoData {
                        for c in codes {
                            if let Ok(o) = meteo::observable::Observable::from_str(&c) {
                                meteo.codes.push(o);
                            }
                        }
                    }
                    obs_code_lines -= 1
                } else {
                    // Observables, 2nd, 3rd.. lines 
                    let codes : Vec<String> = content 
                        .split_ascii_whitespace()
                        .map(|r| r.trim().to_string())
                        .collect(); 
                    if rinex_type == Type::ObservationData {
                        // retrieve correspond system and append codes with new values 
                        let to_retrieve : Vec<Constellation> = match constellation {
                            Some(Constellation::Mixed) => {
                                vec![ // Old OBS Data + Mixed constellation ==> no means to differentiate
                                    Constellation::GPS,
                                    Constellation::Glonass,
                                    Constellation::Galileo,
                                    Constellation::BeiDou,
                                    Constellation::Geo,
                                    Constellation::QZSS,
                                ]
                            },
                            Some(c) => vec![c],
                            None => unreachable!("OBS rinex with no constellation specified"),
                        };
                        for r in to_retrieve {
                            // retrieve map being built
                            if let Some(mut prev) = observation.codes.remove(&r) {
                                // increment obs code map
                                for code in &codes {
                                    prev.push(code.to_string());
                                }
                                observation.codes.insert(r, prev); // (re)insert
                            } 
                        }
                    } else if rinex_type == Type::MeteoData {
                        // simple append, list is simpler
                        for c in codes {
                            if let Ok(o) = meteo::Observable::from_str(&c) {
                                meteo.codes.push(o);
                            }
                        }
                    }
                    obs_code_lines -= 1
                }

            } else if marker.contains("SYS / # / OBS TYPES") {
                // --> observable (V>2 modern fashion)
                if obs_code_lines == 0 {
                    // First line describing observables 
                    let (identifier, rem) = content.split_at(1);
                    let (n, rem) = rem.split_at(5);
                    if let Ok(n) = u8::from_str_radix(n.trim(), 10) {
                        obs_code_lines = num_integer::div_ceil(n, 13); // max. items per line
                    } else {
                        continue // failed to identify # of observables,
                            // we'll continue parsing header section,
                            // record builder won't produce much
                    }
                    let codes : Vec<String> = rem
                        .split_ascii_whitespace()
                        .map(|r| r.trim().to_string())
                        .collect();
                    if let Ok(constell) = Constellation::from_1_letter_code(identifier) {
                        current_code_syst = constell.clone(); // to keep track,
                            // on 2nd and 3rd line, system will not be reminded
                        observation.codes.insert(constell, codes);
                    }
                } else {
                    // 2nd, 3rd.. line of observables 
                    let codes : Vec<String> = content
                        .split_ascii_whitespace()
                        .map(|r| r.trim().to_string())
                        .collect();
                    // increment list with new codes
                    if let Some(list) = observation.codes.get_mut(&current_code_syst) {
                        // increment obs code map
                        for code in codes {
                            list.push(code);
                        }
                    }
                } 
                obs_code_lines -= 1

            } else if marker.contains("ANALYSIS CENTER") {
                let (code, agency) = content.split_at(3);
                clocks = clocks
                    .with_agency(clocks::Agency {
                        code: code.trim().to_string(),
                        name: agency.trim().to_string(),
                    });
            
            } else if marker.contains("# / TYPES OF DATA") {
                let (n, r) = content.split_at(6);
                let n = u8::from_str_radix(n.trim(),10)?;
                let mut rem = r.clone();
                for _ in 0..n {
                    let (code, r) = rem.split_at(6);
                    if let Ok(c) = clocks::DataType::from_str(code.trim()) {
                        clocks.codes.push(c);
                    }
                    rem = r.clone()
                }

            } else if marker.contains("STATION NAME / NUM") {
                let (name, num) = content.split_at(4);
                clocks = clocks
                    .with_ref_station(clocks::Station {
                        id: num.trim().to_string(),
                        name: name.trim().to_string(),
                    });

            } else if marker.contains("STATION CLK REF") {
                clocks = clocks
                    .with_ref_clock(content.trim());
         
            } else if marker.contains("SIGNAL STRENGHT UNIT") {
                //TODO
            
            } else if marker.contains("INTERVAL") {
                let intv_str = content.split_at(20).0.trim();
                if let Ok(interval) = f64::from_str(intv_str) {
                    if interval > 0.0 { // INTERVAL = '0' may exist, in case 
                                    // of Varying TEC map intervals
                        sampling_interval = Some(Duration::from_f64(interval, hifitime::Unit::Second));
                    }
                }

            } else if marker.contains("GLONASS SLOT / FRQ #") {
                //TODO
            } else if marker.contains("GLONASS COD/PHS/BIS") {
                //TODO

            } else if marker.contains("ION ALPHA") { 
                //TODO
                //0.7451D-08 -0.1490D-07 -0.5960D-07  0.1192D-06          ION ALPHA           

            } else if marker.contains("ION BETA") {
                //TODO
                //0.9011D+05 -0.6554D+05 -0.1311D+06  0.4588D+06          ION BETA            
            } else if marker.contains("IONOSPHERIC CORR") {
                // TODO
                // GPSA 0.1025E-07 0.7451E-08 -0.5960E-07 -0.5960E-07
                // GPSB 0.1025E-07 0.7451E-08 -0.5960E-07 -0.5960E-07

            } else if marker.contains("TIME SYSTEM CORR") {
                // TODO
                // GPUT 0.2793967723E-08 0.000000000E+00 147456 1395
            
            } else if marker.contains("TIME SYSTEM ID") {
                let timescale = content.trim();
                if let Ok(ts) = TimeScale::from_str(content.trim()) {
                    clocks = clocks
                        .with_timescale(ts);
                } else {
                    if timescale.eq("GPS") {
                        clocks = clocks
                            .with_timescale(TimeScale::GPST);
                    } else if timescale.eq("GAL") {
                        clocks = clocks
                            .with_timescale(TimeScale::GST);
                    } else if timescale.eq("BDS") {
                        clocks = clocks
                            .with_timescale(TimeScale::BDT);
                    }
                }

            } else if marker.contains("DELTA-UTC") {
                //TODO
                //0.931322574615D-09 0.355271367880D-14   233472     1930 DELTA-UTC: A0,A1,T,W
            
            } else if marker.contains("DESCRIPTION") { // IONEX description
                // <o
                //   if "DESCRIPTION" is to be encountered in other RINEX
                //   we can safely test RinexType here because its already been determined
                ionex = ionex
                    .with_description(content.trim())
            } else if marker.contains("OBSERVABLES USED") { // IONEX observables
                ionex = ionex
                    .with_observables(content.trim())

            } else if marker.contains("ELEVATION CUTOFF") {
                if let Ok(f) = f32::from_str(content.trim()) {
                    ionex = ionex
                        .with_elevation_cutoff(f);
                }
            
            } else if marker.contains("BASE RADIUS") {
                if let Ok(f) = f32::from_str(content.trim()) {
                    ionex = ionex
                        .with_base_radius(f);
                }

            } else if marker.contains("MAPPING FUCTION") {
                if let Ok(mf) = ionex::MappingFunction::from_str(content.trim()) {
                    ionex = ionex
                        .with_mapping_function(mf);
                }

            } else if marker.contains("# OF STATIONS") { // IONEX
                if let Ok(u) = u32::from_str_radix(content.trim(), 10) {
                    ionex = ionex
                        .with_nb_stations(u)
                }
            } else if marker.contains("# OF SATELLITES") { // IONEX
                if let Ok(u) = u32::from_str_radix(content.trim(), 10) {
                    ionex = ionex
                        .with_nb_satellites(u)
                }
            /*
             * Initial TEC map scaling
             */
            } else if marker.contains("EXPONENT") {
                if let Ok(e) = i8::from_str_radix(content.trim(), 10) {
                    ionex = ionex
                        .with_exponent(e);
                }

            /* 
             * Ionex Grid Definition
             */
            } else if marker.contains("HGT1 / HGT2 / DHGT") {
                let items: Vec<&str> = content.split_ascii_whitespace()
                    .collect();
                if items.len() == 3 {
                    if let Ok(start) = f32::from_str(items[0].trim()) {
                        if let Ok(end) = f32::from_str(items[1].trim()) {
                            if let Ok(spacing) = f32::from_str(items[2].trim()) {
                                let grid = match spacing == 0.0 {
                                    true => { // special case, 2D fixed altitude
                                        ionex::GridLinspace { // avoid verifying the Linspace in this case
                                            start,
                                            end,
                                            spacing: 0.0,
                                        }
                                    },
                                    _ => ionex::GridLinspace::new(start, end, spacing)?,
                                };
                                ionex = ionex
                                    .with_altitude_grid(grid);
                            }
                        }
                    }
                }
            } else if marker.contains("LAT1 / LAT2 / DLAT") {
                let items: Vec<&str> = content.split_ascii_whitespace()
                    .collect();
                if items.len() == 3 {
                    if let Ok(start) = f32::from_str(items[0].trim()) {
                        if let Ok(end) = f32::from_str(items[1].trim()) {
                            if let Ok(spacing) = f32::from_str(items[2].trim()) {
                                ionex = ionex
                                    .with_latitude_grid(ionex::GridLinspace::new(start, end, spacing)?); 
                            }
                        }
                    }
                }
            } else if marker.contains("LON1 / LON2 / DLON") {
                let items: Vec<&str> = content.split_ascii_whitespace()
                    .collect();
                if items.len() == 3 {
                    if let Ok(start) = f32::from_str(items[0].trim()) {
                        if let Ok(end) = f32::from_str(items[1].trim()) {
                            if let Ok(spacing) = f32::from_str(items[2].trim()) {
                                ionex = ionex
                                    .with_longitude_grid(ionex::GridLinspace::new(start, end, spacing)?);
                            }
                        }
                    }
                }
            } else if marker.contains("PRN / BIAS / RMS") {
                // differential PR code analysis
                //TODO
            }
        }

        Ok(Header{
            version: version,
            rinex_type,
            constellation,
            comments,
            program,
            run_by,
            date,
            station,
            station_id,
            agency,
            observer,
            license,
            doi,
            station_url,
            marker_type,
            rcvr, 
            leap,
            coords: coords,
            wavelengths: None,
            gps_utc_delta: None,
            sampling_interval,
            data_scaling: None,
            rcvr_antenna,
            sv_antenna,
            // RINEX specific
            obs: {
                if rinex_type == Type::ObservationData {
                    Some(observation)
                } else {
                    None
                }
            },
            meteo: {
                if rinex_type == Type::MeteoData {
                    Some(meteo)
                } else {
                    None
                }
            },
            clocks: {
                if rinex_type == Type::ClockData {
                    Some(clocks)
                } else {
                    None
                }
            },
            ionex: {
                if rinex_type == Type::IonosphereMaps {
                    Some(ionex)
                } else {
                    None
                }
            },
            antex: {
                if rinex_type == Type::AntennaData {
                    Some(antex)   
                } else {
                    None
                }
            },
        })
    }

    /// Combines self and rhs header into a new header.
    /// Self's attribute are always preferred.
    /// Behavior:
    ///  - self's attributes are always preferred (in case of unique attributes)
    ///  - observables are concatenated
    /// This fails if :
    ///  - RINEX types do not match
    ///  - IONEX: map dimensions do not match and grid definitions do not strictly match
    pub fn merge(&self, header: &Self) -> Result<Self, merge::Error> {
        if self.rinex_type != header.rinex_type {
            return Err(merge::Error::FileTypeMismatch);
        }
        if self.rinex_type == Type::IonosphereMaps {
            if let Some(i0) = &self.ionex {
                if let Some(i1) = &header.ionex {
                    if i0.map_dimension != i1.map_dimension {
                        panic!("can only merge ionex files with identical map dimensions")
                    }
                }
            }
        }
        Ok(Self {
            version: { // retains oldest rev
                if self.version < header.version {
                    self.version.clone()
                } else {
                    header.version.clone()
                }
            },
            rinex_type: self.rinex_type.clone(),
            comments: {
                self.comments.clone() //TODO: append rhs too!
            },
            leap: {
                if let Some(leap) = self.leap {
                    Some(leap.clone())
                } else if let Some(leap) = header.leap {
                    Some(leap.clone())
                } else {
                    None
                }
            },
            run_by: self.run_by.clone(),
            program: self.program.clone(),
            observer: self.observer.clone(),
            date: self.date.clone(),
            station: self.station.clone(),
            station_id: self.station_id.clone(),
            station_url: self.station_url.clone(),
            agency: self.agency.clone(),
            license: self.license.clone(),
            doi: self.doi.clone(),
            marker_type: {
                if let Some(mtype) = &self.marker_type {
                    Some(mtype.clone())
                } else if let Some(mtype) = &header.marker_type {
                    Some(mtype.clone())
                } else {
                    None
                }
            },
            gps_utc_delta: {
                if let Some(d) = self.gps_utc_delta {
                    Some(d)
                } else if let Some(d) = header.gps_utc_delta {
                    Some(d)
                } else {
                    None
                }
            },
            data_scaling: {
                if let Some(d) = self.data_scaling {
                    Some(d)
                } else if let Some(d) = header.data_scaling {
                    Some(d)
                } else {
                    None
                }
            },
            constellation: {
                if let Some(c0) = self.constellation {
                    if let Some(c1) = header.constellation {
                        if c0 != c1 {
                            Some(Constellation::Mixed)
                        } else {
                            Some(c0.clone())
                        }
                    } else {
                        Some(c0.clone())
                    }
                } else if let Some(constellation) = header.constellation {
                    Some(constellation.clone())
                } else {
                    None
                }
            },
            rcvr: {
                if let Some(rcvr) = &self.rcvr {
                    Some(rcvr.clone())
                } else if let Some(rcvr) = &header.rcvr {
                    Some(rcvr.clone())
                } else {
                    None
                }
            },
            rcvr_antenna: {
                if let Some(a) = &self.rcvr_antenna {
                    Some(a.clone())
                } else if let Some(a) = &header.rcvr_antenna {
                    Some(a.clone())
                } else {
                    None
                }
            },
            sv_antenna: {
                if let Some(a) = &self.sv_antenna {
                    Some(a.clone())
                } else if let Some(a) = &header.sv_antenna {
                    Some(a.clone())
                } else {
                    None
                }
            },
            wavelengths: {
                if let Some(wv) = &self.wavelengths {
                    Some(wv.clone())
                } else if let Some(wv) = &header.wavelengths {
                    Some(wv.clone())
                } else {
                    None
                }
            },
            sampling_interval: {
                if let Some(interval) = self.sampling_interval {
                    Some(interval.clone())
                } else if let Some(interval) = header.sampling_interval {
                    Some(interval.clone())
                } else {
                    None
                }
            },
            coords: {
                if let Some(coords) = &self.coords {
                    Some(coords.clone())
                } else if let Some(coords) = &header.coords {
                    Some(coords.clone())
                } else {
                    None
                }
            },
            obs: {
                if let Some(d0) = &self.obs {
                    if let Some(d1) = &header.obs {
                        Some(observation::HeaderFields {
                            crinex: d0.crinex.clone(),
                            codes: {
                                let mut map = d0.codes.clone();
                                for (constellation, obscodes) in d1.codes.iter() {
                                    if let Some(codes) = map.get_mut(&constellation) {
                                        for obs in obscodes {
                                            if !codes.contains(&obs) {
                                                codes.push(obs.clone());
                                            }
                                        }
                                    } else {
                                        map.insert(constellation.clone(), obscodes.clone());
                                    }
                                }
                                map
                            },
                            clock_offset_applied: d0.clock_offset_applied && d1.clock_offset_applied,
                            scalings: HashMap::new(), //TODO
                            dcb_compensations: Vec::new(), //TODO
                        })
                    } else {
                        Some(d0.clone())
                    }
                } else if let Some(data) = &header.obs {
                    Some(data.clone())
                } else {
                    None
                }
            },
            meteo: {
                if let Some(m0) = &self.meteo {
                    if let Some(m1) = &header.meteo {
                        Some(meteo::HeaderFields {
                            sensors: {
                                let mut sensors = m0.sensors.clone();
                                for sens in m1.sensors.iter() {
                                    if !sensors.contains(&sens) {
                                        sensors.push(sens.clone())
                                    }
                                }
                                sensors
                            },
                            codes: {
                                let mut observables = m0.codes.clone();
                                for obs in m1.codes.iter() {
                                    if !observables.contains(&obs) {
                                        observables.push(obs.clone())
                                    }
                                }
                                observables
                            },
                        })
                    } else {
                        Some(m0.clone())
                    }
                } else if let Some(meteo) = &header.meteo {
                    Some(meteo.clone())
                } else {
                    None
                }
            },
            clocks: {
                if let Some(d0) = &self.clocks {
                    if let Some(d1) = &header.clocks {
                        Some(clocks::HeaderFields {
                            codes: {
                                let mut codes = d0.codes.clone();
                                for code in d1.codes.iter() {
                                    if !codes.contains(&code) {
                                        codes.push(code.clone())
                                    }
                                }
                                codes
                            },
                            agency: {
                                if let Some(agency) = &d0.agency {
                                    Some(agency.clone())
                                } else if let Some(agency) = &d1.agency {
                                    Some(agency.clone())
                                } else {
                                    None
                                }
                            },
                            station: {
                                if let Some(station) = &d0.station {
                                    Some(station.clone())
                                } else if let Some(station) = &d1.station {
                                    Some(station.clone())
                                } else {
                                    None
                                }
                            },
                            clock_ref: {
                                if let Some(clk) = &d0.clock_ref {
                                    Some(clk.clone())
                                } else if let Some(clk) = &d1.clock_ref {
                                    Some(clk.clone())
                                } else {
                                    None
                                }
                            },
                            timescale: {
                                if let Some(ts) = &d0.timescale {
                                    Some(ts.clone())
                                } else if let Some(ts) = &d1.timescale {
                                    Some(ts.clone())
                                } else {
                                    None
                                }
                            },
                        })
                    } else {
                        Some(d0.clone())
                    }
                } else if let Some(d1) = &header.clocks {
                    Some(d1.clone())
                } else {
                    None
                }
            },
            antex: {
                if let Some(d0) = &self.antex {
                    Some(d0.clone())
                } else if let Some(data) = &header.antex {
                    Some(data.clone())
                } else {
                    None
                }
            },
            ionex: {
                if let Some(d0) = &self.ionex {
                    if let Some(d1) = &header.ionex {
                        Some(ionex::HeaderFields {
                            reference: d0.reference.clone(),
                            description: {
                                if let Some(description) = &d0.description {
                                    Some(description.clone())
                                } else if let Some(description) = &d1.description {
                                    Some(description.clone())
                                } else {
                                    None
                                }
                            },
                            exponent: std::cmp::min(d0.exponent, d1.exponent), // TODO: this is not correct,
                            mapping: {
                                if let Some(map) = &d0.mapping {
                                    Some(map.clone())
                                } else if let Some(map) = &d1.mapping {
                                    Some(map.clone())
                                } else {
                                    None
                                }
                            },
                            map_dimension: d0.map_dimension,
                            base_radius: d0.base_radius,
                            grid: d0.grid.clone(),
                            elevation_cutoff: d0.elevation_cutoff,
                            observables: {
                                if let Some(obs) = &d0.observables {
                                    Some(obs.clone())
                                } else if let Some(obs) = &d1.observables {
                                    Some(obs.clone())
                                } else {
                                    None
                                }
                            },
                            nb_stations: std::cmp::max(d0.nb_stations, d1.nb_stations),
                            nb_satellites: std::cmp::max(d0.nb_satellites, d1.nb_satellites),
                            dcbs: {
                                let mut dcbs = d0.dcbs.clone();
                                for (b, dcb) in &d1.dcbs {
                                    dcbs.insert(b.clone(), *dcb);
                                }
                                dcbs
                            },
                        })
                    } else {
                        Some(d0.clone())
                    }
                } else if let Some(d1) = &header.ionex {
                    Some(d1.clone())
                } else {
                    None
                }
            },
        })
    }
    
    /// Returns true if self is a `Compressed RINEX`
    pub fn is_crinex(&self) -> bool { 
        if let Some(obs) = &self.obs {
            obs.crinex.is_some()
        } else {
            false
        }
    }

    /// Creates a Basic Header structure
    /// for Mixed Constellation Navigation RINEX
    pub fn basic_nav() -> Self {
        Self::default()
            .with_type(Type::NavigationData)
            .with_constellation(Constellation::Mixed)
    }
    
    /// Creates a Basic Header structure
    /// for Mixed Constellation Observation RINEX
    pub fn basic_obs() -> Self {
        Self::default()
            .with_type(Type::ObservationData)
            .with_constellation(Constellation::Mixed)
    }

    /// Creates Basic Header structure
    /// for Compact RINEX with Mixed Constellation context
    pub fn basic_crinex() -> Self {
        Self::default()
            .with_type(Type::ObservationData)
            .with_constellation(Constellation::Mixed)
            .with_crinex(Crinex::default())
    }

    /// Returns Header structure with specific RINEX revision
    pub fn with_version(&self, version: Version) -> Self {
        let mut s = self.clone();
        s.version = version;
        s
    }

    /// Returns Header structure with desired RINEX type
    pub fn with_type(&self, t: Type) -> Self {
        let mut s = self.clone();
        s.rinex_type = t;
        s
    } 

    /// Adds general information to Self
    pub fn with_general_infos (&self, program: &str, run_by: &str, agency: &str) -> Self {
        let mut s = self.clone();
        s.program = program.to_string();
        s.run_by = run_by.to_string();
        s.agency = agency.to_string();
        s
    }

    /// Adds crinex generation attributes to self,
    /// has no effect if this is not an Observation Data header.
    pub fn with_crinex (&self, c: Crinex) -> Self {
        let mut s = self.clone();
        if let Some(ref mut obs) = s.obs {
            obs.crinex = Some(c)
        }
        s
    }

    /// Adds receiver information to self
    pub fn with_receiver(&self, r: Rcvr) -> Self {
        let mut s = self.clone();
        s.rcvr = Some(r);
        s
    }
    
    /// Sets Receiver Antenna information
    pub fn with_receiver_antenna(&self, a: Antenna) -> Self {
        let mut s = self.clone();
        s.rcvr_antenna = Some(a);
        s
    }

    /// Adds desired constellation to Self
    pub fn with_constellation (&self, c: Constellation) -> Self {
        let mut s = self.clone();
        s.constellation = Some(c);
        s
    }

    /// adds comments to Self
    pub fn with_comments (&self, c: Vec<String>) -> Self {
        let mut s = self.clone();
        s.comments = c.clone();
        s
    }
    
    pub fn with_observation_fields (&self, fields: observation::HeaderFields) -> Self {
        let mut s = self.clone();
        s.obs = Some(fields);
        s
    }
}

impl std::fmt::Display for Header {
    /// `Header` formatter, mainly for RINEX file production purposes
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // start with CRINEX attributes, if need be
        if let Some(obs) = &self.obs {
            if let Some(crinex) = &obs.crinex {
                write!(f, "{}\n", crinex)?;
            }
        }
        // RINEX VERSION / TYPE 
        write!(f, "{:6}.{:02}           ", self.version.major, self.version.minor)?;
        match self.rinex_type {
            Type::NavigationData => {
                match self.constellation {
                    Some(Constellation::Glonass) => {
                        // Glonass Special case
                        write!(f,"{:<20}", "G: GLONASS NAV DATA")?;
                        write!(f,"{:<20}", "")?;
                        write!(f,"{}", "RINEX VERSION / TYPE\n")?
                    },
                    Some(c) => {
                        write!(f,"{:<20}", "NAVIGATION DATA")?;
                        write!(f,"{:<20}", c.to_1_letter_code())?;
                        write!(f,"{:<20}", "RINEX VERSION / TYPE\n")?
                    },
                    _ => panic!("constellation must be specified when formatting a NavigationData") 
                }
            },
            Type::ObservationData => {
                match self.constellation {
                    Some(c) => {
                        write!(f,"{:<20}", "OBSERVATION DATA")?;
                        write!(f,"{:<20}", c.to_1_letter_code())?;
                        write!(f,"{:<20}", "RINEX VERSION / TYPE\n")?
                    },
                    _ => panic!("constellation must be specified when formatting ObservationData")
                }
            },
            Type::MeteoData => {
                write!(f,"{:<20}", "METEOROLOGICAL DATA")?;
                write!(f,"{:<20}", "")?;
                write!(f,"{:<20}", "RINEX VERSION / TYPE\n")?;
            },
            Type::ClockData => {
                write!(f,"{:<20}", "CLOCK DATA")?;
                write!(f,"{:<20}", "")?;
                write!(f,"{:<20}", "RINEX VERSION / TYPE\n")?;
            },
            Type::AntennaData => todo!(),
            Type::IonosphereMaps => todo!(),
        }
        // COMMENTS 
        for comment in self.comments.iter() {
            write!(f, "{:<60}", comment)?;
            write!(f, "COMMENT\n")?
        }
        // PGM / RUN BY / DATE
        write!(f, "{:<20}", self.program)?;
        write!(f, "{:<20}", self.run_by)?;
        write!(f, "{:<20}", self.date)?; //TODO
        write!(f, "{}", "PGM / RUN BY / DATE\n")?; 
        // OBSERVER / AGENCY
        write!(f, "{:<20}", self.observer)?;
        write!(f, "{:<40}", self.agency)?;
        write!(f, "OBSERVER / AGENCY\n")?; 
        // MARKER NAME
        if self.station.len() > 0 {
            write!(f, "{:<20}", self.station)?;
            write!(f, "{:<40}", " ")?;
            write!(f, "{}", "MARKER NAME\n")?;
        }
        // MARKER NUMBER
        if self.station_id.len() > 0 { // has been parsed
            write!(f, "{:<20}", self.station_id)?;
            write!(f, "{:<40}", " ")?;
            write!(f, "{}", "MARKER NUMBER\n")?;
        }
        // ANT
        if let Some(antenna) = &self.rcvr_antenna {
            write!(f, "{:<20}", antenna.model)?;
            write!(f, "{:<40}", antenna.sn)?;
            write!(f, "{}", "ANT # / TYPE\n")?;
            if let Some(coords) = &antenna.coords {
                write!(f, "{:14.4}", coords.x)?;
                write!(f, "{:14.4}", coords.y)?;
                write!(f, "{:14.4}", coords.z)?;
                write!(f, "{}", "APPROX POSITION XYZ\n")?
            }
            if let Some(h) = &antenna.height {
                write!(f, "{:14.4}", h)?;
                if let Some(e) = &antenna.eastern {
                    write!(f, "{:14.4}", e)?;
                } else {
                    write!(f, "{:14.4}", 0.0)?;
                }
                if let Some(n) = &antenna.northern {
                    write!(f, "{:14.4}", n)?;
                } else {
                    write!(f, "{:14.4}", 0.0)?;
                }
                write!(f, "{:18}", "")?;
                write!(f, "{}", "ANTENNA: DELTA H/E/N\n")?
            }
        }
        // RCVR
        if let Some(rcvr) = &self.rcvr {
            write!(f, "{:<20}", rcvr.sn)?; 
            write!(f, "{:<20}", rcvr.model)?; 
            write!(f, "{:<20}", rcvr.firmware)?;
            write!(f, "REC # / TYPE / VERS\n")?
        }
        // INTERVAL
        if let Some(interval) = &self.sampling_interval {
            write!(f, "{:10.3}", interval)?;
            write!(f, "{:<50}", "")?;
            write!(f, "INTERVAL\n")?
        }
        // List of Observables
        match self.rinex_type {
            Type::ObservationData => {
                if let Some(obs) = &self.obs {
                    match self.version.major {
                        1|2 => { // old revisions
                            for (_, observables) in obs.codes.iter() {
                                write!(f, "{:6}", observables.len())?; 
                                let mut line = String::new();
                                for i in 0..observables.len() {
                                    if (i % 9) == 0 && i > 0 {
                                        line.push_str("# / TYPES OF OBSERV\n");
                                        write!(f, "{}", line)?;
                                        line.clear();
                                        line.push_str(&format!("{:6}", "")); // tab
                                    }
                                    line.push_str(&format!("{:>6}", observables[i]));
                                }
                                if line.len() > 0 { // residues
                                    if observables.len() > 9 {
                                        line.push_str(&format!("{:<width$}", "", width=60-line.len()));
                                    } else {
                                        line.push_str(&format!("{:<width$}", "", width=54-line.len()));
                                    }
                                    line.push_str("# / TYPES OF OBSERV\n");
                                    //line.push_str(&format!("{:>width$}", "# / TYPES OF OBSERV\n", width=74-line.len()));
                                    write!(f, "{}", line)?
                                }
                                break ; // run only once, <=> for 1 constellation
                            }
                        },
                        _ => { // modern revisions
                            for (constell, codes) in obs.codes.iter() {
                                let mut line = format!("{:<4}", constell.to_1_letter_code());
                                line.push_str(&format!("{:2}", codes.len())); 
                                for i in 0..codes.len() {
                                    if (i+1)%14 == 0 {
                                        line.push_str(&format!("{:<width$}", "", width=60-line.len()));
                                        line.push_str("SYS / # / OBS TYPES\n"); 
                                        write!(f, "{}", line)?;
                                        line.clear();
                                        line.push_str(&format!("{:<6}", ""));
                                    }
                                    line.push_str(&format!(" {}", codes[i]))
                                }
                                line.push_str(&format!("{:<width$}", "", width=60-line.len()));
                                line.push_str("SYS / # / OBS TYPES\n"); 
                                write!(f, "{}", line)?
                            }
                        },
                    }
                }
            }, //ObservationData
            Type::MeteoData => {
                if let Some(obs) = &self.meteo {
                    write!(f, "{:6}", obs.codes.len())?; 
                    let mut line = String::new();
                    for i in 0..obs.codes.len() {
                        if (i % 9) == 0 && i > 0 {
                            line.push_str("# / TYPES OF OBSERV\n");
                            write!(f, "{}", line)?;
                            line.clear();
                            line.push_str(&format!("{:6}", "")); // tab
                        }
                        line.push_str(&format!("{:>6}", obs.codes[i]));
                    }
                    if line.len() > 0 { // residues
                        if obs.codes.len() > 9 {
                            line.push_str(&format!("{:<width$}", "", width=60-line.len()));
                        } else {
                            line.push_str(&format!("{:<width$}", "", width=54-line.len()));
                        }
                        line.push_str("# / TYPES OF OBSERV\n");
                    }
                    write!(f, "{}", line)?
                }
            },//meteo data
            _ => {},
        }
        // Must take place after list of Observables:
        //TODO: scale factor, if any
        //TODO: DCBS compensation, if any
        //TODO: PCVs compensation, if any
        // LEAP
        if let Some(leap) = &self.leap {
            let mut line = String::new();
            line.push_str(&format!("{:6}", leap.leap));
            if let Some(delta) = &leap.delta_tls {
                line.push_str(&format!("{:6}", delta));
                line.push_str(&format!("{:6}", leap.week.unwrap_or(0)));
                line.push_str(&format!("{:6}", leap.day.unwrap_or(0)));
                if let Some(timescale) = &leap.timescale {
                    line.push_str(&format!("{:<10}", timescale)); 
                } else {
                    line.push_str(&format!("{:<10}", ""));
                }
            }
            line.push_str(&format!("{:>width$}", "LEAP SECONDS\n", width=73-line.len()));
            write!(f, "{}", line)?
        }
        // Custom Meteo fields
        if let Some(meteo) = &self.meteo {
            let sensors = &meteo.sensors;
            for sensor in sensors {
                write!(f, "{}", sensor)?
            }
        }
        // Custom Clock fields
        if let Some(clocks) = &self.clocks {
            // Types of data: is the equivalent of Observation codes
            write!(f, "{:6}", clocks.codes.len())?;
            for code in &clocks.codes {
                write!(f, "    {}", code)?;
            }
            write!(f, "{:>width$}\n", "# / TYPES OF DATA\n", width=80-6-6*clocks.codes.len()-2)?;

            // possible timescale
            if let Some(ts) = clocks.timescale {
                write!(f, "   {:x}                                                     TIME SYSTEM ID\n", ts)?; 
            }
            // possible reference agency 
            if let Some(agency) = &clocks.agency {
                write!(f, "{:<5} ", agency.code)?;
                write!(f, "{}", agency.name)?;
                write!(f, "ANALYSIS CENTER\n")?;
            }
            // possible reference clock information
        }
        // Custom IONEX fields
        if let Some(ionex) = &self.ionex {
            //TODO:
            //  EPOCH OF FIRST and LAST MAP
            //   with epoch::format(Ionex)
            let _ = write!(f, "{:6}           MAP DIMENSION\n", ionex.map_dimension);
            let h = &ionex.grid.height;
            let _ = write!(f, "{} {}  {}     HGT1 / HGT2 / DHGT\n", h.start, h.end, h.spacing);
            let lat = &ionex.grid.latitude;
            let _ = write!(f, "{} {}  {}     LAT1 / LON2 / DLAT\n", lat.start, lat.end, lat.spacing);
            let lon = &ionex.grid.longitude;
            let _ = write!(f, "{} {}  {}     LON1 / LON2 / DLON\n", lon.start, lon.end, lon.spacing);
            let _ = write!(f, "{}         ELEVATION CUTOFF\n", ionex.elevation_cutoff);
            if let Some(func) = &ionex.mapping {
                let _ = write!(f, "{:?}         MAPPING FUNCTION\n", func);
            } else {
                let _ = write!(f, "NONE         MAPPING FUNCTION\n");
            }
            let _ = write!(f, "{}               EXPONENT\n", ionex.exponent);
            if let Some(desc) = &ionex.description {
                for line in 0..desc.len() / 60 {
                    let max = std::cmp::min((line+1)*60, desc.len());
                    let _ = write!(f, "{}                COMMENT\n", &desc[line*60..max]);
                }
            }
        }
        // END OF HEADER
        write!(f, "{:>74}", "END OF HEADER\n")
    }
}

impl Merge<Header> for Header {
    /// Merges `rhs` into `Self` without mutable access, at the expense of memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        if self.rinex_type != rhs.rinex_type {
            return Err(merge::Error::FileTypeMismatch);
        }
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self` in place
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        if self.rinex_type != rhs.rinex_type {
            return Err(merge::Error::FileTypeMismatch);
        }

        let (a_cst, b_cst) = (self.constellation, rhs.constellation);
        if a_cst != b_cst {
            // <=> Constellation "upgrade"
            self.constellation = Some(Constellation::Mixed)
        }
        
        // retain oldest revision
        let (a_rev, b_rev) = (self.version, rhs.version);
        self.version = std::cmp::min(a_rev, b_rev);

        merge::merge_mut_vec(&mut self.comments, &rhs.comments);
        merge::merge_mut_option(&mut self.marker_type, &rhs.marker_type);
        merge::merge_mut_option(&mut self.sampling_interval, &rhs.sampling_interval);
        merge::merge_mut_option(&mut self.license, &rhs.license);
        merge::merge_mut_option(&mut self.data_scaling, &rhs.data_scaling);
        merge::merge_mut_option(&mut self.doi, &rhs.doi);
        merge::merge_mut_option(&mut self.leap, &rhs.leap);
        merge::merge_mut_option(&mut self.gps_utc_delta, &rhs.gps_utc_delta);
        merge::merge_mut_option(&mut self.rcvr, &rhs.rcvr);
        merge::merge_mut_option(&mut self.rcvr_antenna, &rhs.rcvr_antenna);
        merge::merge_mut_option(&mut self.sv_antenna, &rhs.sv_antenna);
        merge::merge_mut_option(&mut self.coords, &rhs.coords);
        merge::merge_mut_option(&mut self.wavelengths, &rhs.wavelengths);

        // RINEX specific operation
        if let Some(lhs) = &mut self.antex {
            if let Some(rhs) = &rhs.antex {
                // ANTEX records can only be merged together
                // if they have the same type of inner phase data
                let mut mixed_antex = lhs.pcv.is_relative() && !rhs.pcv.is_relative();
                mixed_antex |= !lhs.pcv.is_relative() && rhs.pcv.is_relative();
                if mixed_antex {
                    return Err(merge::Error::AntexAbsoluteRelativeMismatch);
                }
                merge::merge_mut_option(&mut lhs.reference_sn, &rhs.reference_sn);
            }
        }
        if let Some(lhs) = &mut self.clocks {
            if let Some(rhs) = &rhs.clocks {
                merge::merge_mut_unique_vec(&mut lhs.codes, &rhs.codes);
                merge::merge_mut_option(&mut lhs.agency, &rhs.agency);
                merge::merge_mut_option(&mut lhs.station, &rhs.station);
                merge::merge_mut_option(&mut lhs.clock_ref, &rhs.clock_ref);
                merge::merge_mut_option(&mut lhs.timescale, &rhs.timescale);
            }
        }
        if let Some(lhs) = &mut self.obs {
            if let Some(rhs) = &rhs.obs {
                merge::merge_mut_option(&mut lhs.crinex, &rhs.crinex);
                merge::merge_mut_unique_map2d(&mut lhs.codes, &rhs.codes);
                lhs.clock_offset_applied |= rhs.clock_offset_applied;
            }
        }
        if let Some(lhs) = &mut self.meteo {
            if let Some(rhs) = &rhs.meteo {
                merge::merge_mut_unique_vec(&mut lhs.codes, &rhs.codes);
                merge::merge_mut_unique_vec(&mut lhs.sensors, &rhs.sensors);
            }
        }
        if let Some(lhs) = &mut self.ionex {
            if let Some(rhs) = &rhs.ionex {
                if lhs.reference != rhs.reference {
                    return Err(merge::Error::IonexReferenceMismatch);
                }
                if lhs.grid != rhs.grid {
                    return Err(merge::Error::IonexMapGridMismatch);
                }
                if lhs.map_dimension != rhs.map_dimension {
                    return Err(merge::Error::IonexMapDimensionsMismatch);
                }
                if lhs.base_radius != rhs.base_radius {
                    return Err(merge::Error::IonexBaseRadiusMismatch);
                }
                merge::merge_mut_option(&mut lhs.description, &rhs.description);
                merge::merge_mut_option(&mut lhs.mapping, &rhs.mapping);
                if lhs.elevation_cutoff == 0.0 { // means "unknown"
                    lhs.elevation_cutoff = rhs.elevation_cutoff; // => overwrite in this case
                }
                merge::merge_mut_option(&mut lhs.observables, &rhs.observables);
                lhs.nb_stations = std::cmp::max(lhs.nb_stations, rhs.nb_stations);
                lhs.nb_satellites = std::cmp::max(lhs.nb_satellites, rhs.nb_satellites);
                for (b, dcb) in &rhs.dcbs {
                    lhs.dcbs.insert(b.clone(), *dcb);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_from_b_fmt_month() {
        assert_eq!(from_b_fmt_month!("Jan"), 1);
        assert_eq!(from_b_fmt_month!("Feb"), 2);
        assert_eq!(from_b_fmt_month!("Mar"), 3);
        assert_eq!(from_b_fmt_month!("Dec"), 12);
        assert_eq!(from_b_fmt_month!("Nov"), 11);
    }
}
