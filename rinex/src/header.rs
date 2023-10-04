//! Describes a `RINEX` header, includes
//! rinex header parser and associated methods
use super::*;
use crate::{
    antex, clocks,
    clocks::{ClockAnalysisAgency, ClockDataType},
    ground_position::GroundPosition,
    hardware::{Antenna, Rcvr, SvAntenna},
    ionex, leap, meteo, observation,
    observation::Crinex,
    reader::BufferedReader,
    types::Type,
    version::Version,
    Observable,
};

use hifitime::Epoch;
use std::io::prelude::*;
use std::str::FromStr;
use strum_macros::EnumString;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Eq, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MarkerType {
    /// Earth fixed & high precision
    #[strum(serialize = "GEODETIC", serialize = "Geodetic")]
    #[default]
    Geodetic,
    /// Earth fixed & low precision
    #[strum(serialize = "NON GEODETIC", serialize = "NonGeodetic")]
    NonGeodetic,
    /// Generated from network
    #[strum(serialize = "NON PHYSICAL", serialize = "NonPhysical")]
    NonPhysical,
    /// Orbiting space vehicle
    #[strum(serialize = "SPACE BORNE", serialize = "Spaceborne")]
    Spaceborne,
    /// Aircraft, balloon..
    #[strum(serialize = "AIR BORNE", serialize = "Airborne")]
    Airborne,
    /// Mobile water craft
    #[strum(serialize = "WATER CRAFT", serialize = "Watercraft")]
    Watercraft,
    /// Mobile terrestrial vehicle
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

/// DCB compensation description
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DcbCompensation {
    /// Program used for DCBs evaluation and compensation
    pub program: String,
    /// Constellation to which this compensation applies to
    pub constellation: Constellation,
    /// URL: source of corrections
    pub url: String,
}

/// PCV compensation description
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PcvCompensation {
    /// Program used for PCVs evaluation and compensation
    pub program: String,
    /// Constellation to which this compensation applies to
    pub constellation: Constellation,
    /// URL: source of corrections
    pub url: String,
}

/// Describes `RINEX` file header
#[derive(Clone, Debug, Default, PartialEq)]
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
    pub comments: Vec<String>,
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
    /// Glonass FDMA channels
    pub glo_channels: HashMap<Sv, i8>,
    /// optionnal leap seconds infos
    pub leap: Option<leap::Leap>,
    // /// Optionnal system time correction
    // pub time_corrections: Option<gnss_time::Correction>,
    /// Station approximate coordinates
    pub ground_position: Option<GroundPosition>,
    /// Optionnal observation wavelengths
    pub wavelengths: Option<(u32, u32)>,
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
    /// Optionnal Vehicle Antenna information,
    /// attached to a specifid Sv, only exists in ANTEX records
    #[cfg_attr(feature = "serde", serde(default))]
    pub sv_antenna: Option<SvAntenna>,
    /// Possible DCBs compensation information
    pub dcb_compensations: Vec<DcbCompensation>,
    /// Possible PCVs compensation information
    pub pcv_compensations: Vec<PcvCompensation>,
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
pub enum ParsingError {
    #[error("failed to parse version from \"{0}\"")]
    VersionParsing(String),
    #[error("version \"{0}\" is not supported")]
    VersionNotSupported(String),
    #[error("unknown RINEX type \"{0}\"")]
    TypeParsing(String),
    #[error("unknown marker type \"{0}\"")]
    MarkerType(String),
    #[error("failed to parse observable")]
    ObservableParsing(#[from] observable::ParsingError),
    #[error("constellation parsing error")]
    ConstellationParsing(#[from] constellation::ParsingError),
    #[error("timescale parsing error")]
    TimescaleParsing(String),
    #[error("failed to parse \"{0}\" coordinates from \"{1}\"")]
    CoordinatesParsing(String, String),
    #[error("failed to parse leap from \"{0}\"")]
    LeapParsingError(#[from] leap::Error),
    #[error("failed to parse antenna / receiver infos")]
    AntennaRcvrError(#[from] std::io::Error),
    #[error("failed to parse ANTEX fields")]
    AntexParsingError(#[from] antex::record::Error),
    #[error("failed to parse PCV field")]
    ParsePcvError(#[from] antex::pcv::Error),
    #[error("unknown ionex reference")]
    UnknownReferenceIonex(#[from] ionex::system::Error),
    #[error("invalid crinex header \"{0}\": \"{1}\"")]
    CrinexHeader(String, String),
    #[error("failed to parse datetime {0} field from \"{1}\"")]
    DateTimeParsing(String, String),
    #[error("failed to parse {0} integer value from \"{1}\"")]
    ParseIntError(String, String),
    #[error("failed to parse {0} float value from \"{1}\"")]
    ParseFloatError(String, String),
    #[error("failed to parse ionex grid {0} from \"{1}\"")]
    InvalidIonexGrid(String, String),
    #[error("invalid ionex grid definition")]
    InvalidIonexGridDefinition(#[from] ionex::grid::Error),
}

fn parse_formatted_month(content: &str) -> Result<u8, ParsingError> {
    match content {
        "Jan" => Ok(1),
        "Feb" => Ok(2),
        "Mar" => Ok(3),
        "Apr" => Ok(4),
        "May" => Ok(5),
        "Jun" => Ok(6),
        "Jul" => Ok(7),
        "Aug" => Ok(8),
        "Sep" => Ok(9),
        "Oct" => Ok(10),
        "Nov" => Ok(11),
        "Dec" => Ok(12),
        _ => Err(ParsingError::DateTimeParsing(
            String::from("month"),
            content.to_string(),
        )),
    }
}

/*
 * Generates a ParsingError::ParseIntError(x, y)
 */
macro_rules! parse_int_error {
    ($field: expr, $content: expr) => {
        ParsingError::ParseIntError(String::from($field), $content.to_string())
    };
}

/*
 * Generates a ParsingError::ParseFloatError(x, y)
 */
macro_rules! parse_float_error {
    ($field: expr, $content: expr) => {
        ParsingError::ParseFloatError(String::from($field), $content.to_string())
    };
}

/*
 * Generates a ParsingError::InvalidIonexGridError(x, y)
 */
macro_rules! grid_format_error {
    ($field: expr, $content: expr) => {
        ParsingError::InvalidIonexGrid(String::from($field), $content.to_string())
    };
}

impl Header {
    /// Builds a `Header` from stream reader
    pub fn new(reader: &mut BufferedReader) -> Result<Header, ParsingError> {
        let mut rinex_type = Type::default();
        let mut constellation: Option<Constellation> = None;
        let mut version = Version::default();
        let mut comments: Vec<String> = Vec::new();
        let mut program = String::new();
        let mut run_by = String::new();
        let mut date = String::new();
        let mut station = String::new();
        let mut station_id = String::new();
        let mut observer = String::new();
        let mut agency = String::new();
        let mut license: Option<String> = None;
        let mut doi: Option<String> = None;
        let mut station_url = String::new();
        let mut marker_type: Option<MarkerType> = None;
        let mut glo_channels: HashMap<Sv, i8> = HashMap::new();
        let mut rcvr: Option<Rcvr> = None;
        let mut rcvr_antenna: Option<Antenna> = None;
        let mut sv_antenna: Option<SvAntenna> = None;
        let mut leap: Option<leap::Leap> = None;
        let mut sampling_interval: Option<Duration> = None;
        let mut ground_position: Option<GroundPosition> = None;
        let mut dcb_compensations: Vec<DcbCompensation> = Vec::new();
        let mut pcv_compensations: Vec<PcvCompensation> = Vec::new();
        let mut scaling_count = 0_u16;
        // RINEX specific fields
        let mut current_constell: Option<Constellation> = None;
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
                continue; // --> invalid header content
            }
            let (content, marker) = line.split_at(60);
            ///////////////////////////////
            // [0] END OF HEADER
            //     --> done parsing
            ///////////////////////////////
            if marker.trim().eq("END OF HEADER") {
                break;
            }
            ///////////////////////////////
            // [0*] COMMENTS
            ///////////////////////////////
            if marker.trim().eq("COMMENT") {
                // --> storing might be useful
                comments.push(content.trim().to_string());
                continue;

            //////////////////////////////////////
            // [1] CRINEX Special fields
            /////////////////////////////////////
            } else if marker.contains("CRINEX VERS") {
                let version = content.split_at(20).0;
                let version = version.trim();
                let crinex_revision = Version::from_str(version).or(Err(
                    ParsingError::VersionParsing(format!("CRINEX VERS: \"{}\"", version)),
                ))?;

                observation.crinex = Some(Crinex::default().with_version(crinex_revision));
            } else if marker.contains("CRINEX PROG / DATE") {
                let (prog, remainder) = content.split_at(20);
                let (_, remainder) = remainder.split_at(20);
                let date = remainder.split_at(20).0.trim();
                let items: Vec<&str> = date.split_ascii_whitespace().collect();
                if items.len() != 2 {
                    return Err(ParsingError::CrinexHeader(
                        String::from("CRINEX PROG/DATE"),
                        content.to_string(),
                    ));
                }

                let date: Vec<&str> = items[0].split("-").collect();
                let time: Vec<&str> = items[1].split(":").collect();

                let day = date[0].trim();
                let day = u8::from_str_radix(day, 10).or(Err(ParsingError::DateTimeParsing(
                    String::from("day"),
                    day.to_string(),
                )))?;

                let month = date[1].trim();
                let month = parse_formatted_month(month)?;

                let y = date[2].trim();
                let mut y = i32::from_str_radix(y, 10).or(Err(ParsingError::DateTimeParsing(
                    String::from("year"),
                    y.to_string(),
                )))?;

                let h = time[0].trim();
                let h = u8::from_str_radix(h, 10).or(Err(ParsingError::DateTimeParsing(
                    String::from("hour"),
                    h.to_string(),
                )))?;

                let m = time[1].trim();
                let m = u8::from_str_radix(m, 10).or(Err(ParsingError::DateTimeParsing(
                    String::from("minute"),
                    m.to_string(),
                )))?;

                if let Some(crinex) = &mut observation.crinex {
                    y += 2000;
                    let date = Epoch::from_gregorian_utc(y, month, day, h, m, 0, 0);
                    *crinex = crinex.with_prog(prog.trim()).with_date(date);
                }

            ////////////////////////////////////////
            // [2] ANTEX special header
            ////////////////////////////////////////
            } else if marker.contains("ANTEX VERSION / SYST") {
                let (vers, system) = content.split_at(8);
                let vers = vers.trim();
                version = Version::from_str(vers).or(Err(ParsingError::VersionParsing(
                    format!("ANTEX VERSION / SYST: \"{}\"", vers),
                )))?;

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
                let items: Vec<&str> = content.split_ascii_whitespace().collect();
                if items.len() == 2 {
                    // Receiver antenna information
                    // like standard RINEX
                    let (model, rem) = content.split_at(20);
                    let (sn, _) = rem.split_at(20);
                    if let Some(a) = &mut rcvr_antenna {
                        *a = a.with_model(model.trim()).with_serial_number(sn.trim());
                    } else {
                        rcvr_antenna = Some(
                            Antenna::default()
                                .with_model(model.trim())
                                .with_serial_number(sn.trim()),
                        );
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
                            sv_antenna = Some(
                                SvAntenna::default()
                                    .with_sv(sv)
                                    .with_model(model.trim())
                                    .with_cospar(cospar.trim()),
                            );
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

                let vers_str = vers_str.trim();
                version = Version::from_str(vers_str).or(Err(ParsingError::VersionParsing(
                    format!("IONEX VERSION / TYPE : \"{}\"", vers_str),
                )))?;

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
                    constellation = Some(Constellation::Glonass);
                } else if type_str.contains("GPS NAV DATA") {
                    // old GPS NAV: no constellation field
                    constellation = Some(Constellation::GPS);
                } else if type_str.contains("METEOROLOGICAL DATA") {
                    // these files are not tied to a constellation system,
                    // therefore, do not have this field
                } else {
                    // regular files
                    if let Ok(constell) = Constellation::from_str(constell_str.trim()) {
                        constellation = Some(constell);
                    }
                }
                /*
                 * Parse version descriptor
                 */
                let vers = vers.trim();
                version = Version::from_str(vers).or(Err(ParsingError::VersionParsing(
                    format!("RINEX VERSION / TYPE \"{}\"", vers.to_string()),
                )))?;

                if !version.is_supported() {
                    return Err(ParsingError::VersionNotSupported(vers.to_string()));
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
                date = date_str.trim().to_string();
            } else if marker.contains("MARKER NAME") {
                station = content.split_at(20).0.trim().to_string()
            } else if marker.contains("MARKER NUMBER") {
                station_id = content.split_at(20).0.trim().to_string()
            } else if marker.contains("MARKER TYPE") {
                let code = content.split_at(20).0.trim();
                if let Ok(marker) = MarkerType::from_str(code) {
                    marker_type = Some(marker);
                } else {
                    return Err(ParsingError::MarkerType(code.to_string()));
                }
            } else if marker.contains("OBSERVER / AGENCY") {
                let (obs, ag) = content.split_at(20);
                observer = obs.trim().to_string();
                agency = ag.trim().to_string();
            } else if marker.contains("REC # / TYPE / VERS") {
                if let Ok(receiver) = Rcvr::from_str(content) {
                    rcvr = Some(receiver);
                }
            } else if marker.contains("SYS / PCVS APPLIED") {
                let (gnss, rem) = content.split_at(2);
                let (program, rem) = rem.split_at(18);
                let (url, _) = rem.split_at(40);

                let gnss = gnss.trim();
                let gnss = Constellation::from_str(gnss.trim())?;

                let pcv = PcvCompensation {
                    program: {
                        let program = program.trim();
                        if program.eq("") {
                            String::from("Unknown")
                        } else {
                            program.to_string()
                        }
                    },
                    constellation: gnss.clone(),
                    url: {
                        let url = url.trim();
                        if url.eq("") {
                            String::from("Unknown")
                        } else {
                            url.to_string()
                        }
                    },
                };

                pcv_compensations.push(pcv);
            } else if marker.contains("SYS / DCBS APPLIED") {
                let (gnss, rem) = content.split_at(2);
                let (program, rem) = rem.split_at(18);
                let (url, _) = rem.split_at(40);

                let gnss = gnss.trim();
                let gnss = Constellation::from_str(gnss.trim())?;

                let dcb = DcbCompensation {
                    program: {
                        let program = program.trim();
                        if program.eq("") {
                            String::from("Unknown")
                        } else {
                            program.to_string()
                        }
                    },
                    constellation: gnss.clone(),
                    url: {
                        let url = url.trim();
                        if url.eq("") {
                            String::from("Unknown")
                        } else {
                            url.to_string()
                        }
                    },
                };

                dcb_compensations.push(dcb);
            } else if marker.contains("SYS / SCALE FACTOR") {
                //TODO: conclude other lines parsing
                if scaling_count == 0 {
                    // parsing first line
                    let (gnss, rem) = content.split_at(2);
                    let gnss = Constellation::from_str(gnss.trim())?;

                    let (factor, rem) = rem.split_at(6);
                    let factor = factor.trim();
                    let scaling = u16::from_str_radix(factor, 10)
                        .or(Err(parse_int_error!("SYS / SCALE FACTOR", factor)))?;

                    let (_num, rem) = rem.split_at(3);

                    // parse end of line
                    let mut len = rem.len();
                    let mut rem = rem.clone();

                    while len > 0 {
                        let (observable, r) = rem.split_at(4);
                        let observable = Observable::from_str(observable.trim())?;
                        // latch scaling value
                        observation.insert_scaling(gnss, observable, scaling);
                        // continue
                        rem = r.clone();
                        len = rem.len();
                    }
                    scaling_count += 1;
                }
            } else if marker.contains("SENSOR MOD/TYPE/ACC") {
                if let Ok(sensor) = meteo::sensor::Sensor::from_str(content) {
                    meteo.sensors.push(sensor)
                }
            } else if marker.contains("SENSOR POS XYZ/H") {
                /*
                 * Meteo: sensor position information
                 */
                let (x, rem) = content.split_at(14);
                let (y, rem) = rem.split_at(14);
                let (z, rem) = rem.split_at(14);
                let (h, phys) = rem.split_at(14);

                let phys = phys.trim();
                let observable = Observable::from_str(phys)?;

                let x = x.trim();
                let x = f64::from_str(x).or(Err(ParsingError::CoordinatesParsing(
                    String::from("SENSOR POS X"),
                    x.to_string(),
                )))?;

                let y = y.trim();
                let y = f64::from_str(y).or(Err(ParsingError::CoordinatesParsing(
                    String::from("SENSOR POS Y"),
                    y.to_string(),
                )))?;

                let z = z.trim();
                let z = f64::from_str(z).or(Err(ParsingError::CoordinatesParsing(
                    String::from("SENSOR POS Z"),
                    z.to_string(),
                )))?;

                let h = h.trim();
                let h = f64::from_str(h).or(Err(ParsingError::CoordinatesParsing(
                    String::from("SENSOR POS H"),
                    h.to_string(),
                )))?;

                for sensor in meteo.sensors.iter_mut() {
                    if sensor.observable == observable {
                        *sensor = sensor.with_position((x, y, z, h))
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
                let items: Vec<&str> = content.split_ascii_whitespace().collect();
                let x = items[0].trim();
                let x = f64::from_str(x).or(Err(ParsingError::CoordinatesParsing(
                    String::from("APPROX POSITION X"),
                    x.to_string(),
                )))?;

                let y = items[1].trim();
                let y = f64::from_str(y).or(Err(ParsingError::CoordinatesParsing(
                    String::from("APPROX POSITION Y"),
                    y.to_string(),
                )))?;

                let z = items[2].trim();
                let z = f64::from_str(z).or(Err(ParsingError::CoordinatesParsing(
                    String::from("APPROX POSITION Z"),
                    z.to_string(),
                )))?;

                ground_position = Some(GroundPosition::from_ecef_wgs84((x, y, z)));
            } else if marker.contains("ANT # / TYPE") {
                let (model, rem) = content.split_at(20);
                let (sn, _) = rem.split_at(20);
                if let Some(a) = &mut rcvr_antenna {
                    *a = a.with_model(model.trim()).with_serial_number(sn.trim());
                } else {
                    rcvr_antenna = Some(
                        Antenna::default()
                            .with_model(model.trim())
                            .with_serial_number(sn.trim()),
                    );
                }
            } else if marker.contains("ANTENNA: DELTA X/Y/Z") {
                // Antenna Base/Reference Coordinates
                let items: Vec<&str> = content.split_ascii_whitespace().collect();

                let x = items[0].trim();
                let x = f64::from_str(x).or(Err(ParsingError::CoordinatesParsing(
                    String::from("ANTENNA DELTA X"),
                    x.to_string(),
                )))?;

                let y = items[1].trim();
                let y = f64::from_str(y).or(Err(ParsingError::CoordinatesParsing(
                    String::from("ANTENNA DELTA Y"),
                    y.to_string(),
                )))?;

                let z = items[2].trim();
                let z = f64::from_str(z).or(Err(ParsingError::CoordinatesParsing(
                    String::from("ANTENNA DELTA Z"),
                    z.to_string(),
                )))?;

                if let Some(ant) = &mut rcvr_antenna {
                    *ant = ant.with_base_coordinates((x, y, z));
                } else {
                    rcvr_antenna = Some(Antenna::default().with_base_coordinates((x, y, z)));
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
                                rcvr_antenna = Some(
                                    Antenna::default()
                                        .with_height(h)
                                        .with_eastern_component(e)
                                        .with_northern_component(n),
                                );
                            }
                        }
                    }
                }
            } else if marker.contains("ANTENNA: B.SIGHT XYZ") {
                //TODO
            } else if marker.contains("ANTENNA: ZERODIR XYZ") {
                //TODO
            } else if marker.contains("ANTENNA: PHASECENTER") {
                //TODO
            } else if marker.contains("CENTER OF MASS: XYZ") {
                //TODO
            } else if marker.contains("RCV CLOCK OFFS APPL") {
                let value = content.split_at(20).0.trim();
                let n = i32::from_str_radix(value, 10)
                    .or(Err(parse_int_error!("RCV CLOCK OFFS APPL", value)))?;

                observation.clock_offset_applied = n > 0;
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
            } else if marker.contains("TIME OF FIRST OBS") {
                let mut time_of_first_obs = Self::parse_time_of_obs(content)?;
                match constellation {
                    Some(Constellation::Mixed) | None => {},
                    Some(c) => {
                        // in case of OLD RINEX : fixed constellation
                        //  use that information, as it may be omitted in the TIME OF OBS header
                        time_of_first_obs.time_scale = c
                            .timescale()
                            .ok_or(ParsingError::TimescaleParsing(c.to_string()))?;
                    },
                }
                observation = observation.with_time_of_first_obs(time_of_first_obs);
            } else if marker.contains("TIME OF LAST OBS") {
                let mut time_of_last_obs = Self::parse_time_of_obs(content)?;
                match constellation {
                    Some(Constellation::Mixed) | None => {},
                    Some(c) => {
                        // in case of OLD RINEX : fixed constellation
                        //  use that information, as it may be omitted in the TIME OF OBS header
                        time_of_last_obs.time_scale = c
                            .timescale()
                            .ok_or(ParsingError::TimescaleParsing(c.to_string()))?;
                    },
                }
                observation = observation.with_time_of_last_obs(time_of_last_obs);
            } else if marker.contains("TYPES OF OBS") {
                // these observations can serve both Observation & Meteo RINEX
                let (_, content) = content.split_at(6);
                for i in 0..content.len() / 6 {
                    let obscode = &content[i * 6..std::cmp::min((i + 1) * 6, content.len())].trim();
                    if let Ok(observable) = Observable::from_str(obscode) {
                        match constellation {
                            Some(Constellation::Mixed) => {
                                lazy_static! {
                                    static ref KNOWN_CONSTELLS: [Constellation; 6] = [
                                        Constellation::GPS,
                                        Constellation::Glonass,
                                        Constellation::Galileo,
                                        Constellation::BeiDou,
                                        Constellation::QZSS,
                                        Constellation::SBAS,
                                    ];
                                }
                                for c in KNOWN_CONSTELLS.iter() {
                                    if let Some(codes) = observation.codes.get_mut(&c) {
                                        codes.push(observable.clone());
                                    } else {
                                        observation.codes.insert(*c, vec![observable.clone()]);
                                    }
                                }
                            },
                            Some(c) => {
                                if let Some(codes) = observation.codes.get_mut(&c) {
                                    codes.push(observable.clone());
                                } else {
                                    observation.codes.insert(c, vec![observable.clone()]);
                                }
                            },
                            _ => {
                                if rinex_type == Type::MeteoData {
                                    meteo.codes.push(observable);
                                } else {
                                    panic!("can't have \"TYPES OF OBS\" when GNSS definition is missing");
                                }
                            },
                        }
                    }
                }
            } else if marker.contains("SYS / # / OBS TYPES") {
                let (possible_content, content) = content.split_at(6);
                if possible_content.len() > 0 {
                    let code = &possible_content[..1];
                    if let Ok(c) = Constellation::from_str(code) {
                        current_constell = Some(c);
                    }
                }

                if let Some(constell) = current_constell {
                    // system correctly identified
                    for i in 0..content.len() / 4 {
                        let obscode =
                            &content[i * 4..std::cmp::min((i + 1) * 4, content.len())].trim();
                        if let Ok(observable) = Observable::from_str(obscode) {
                            if obscode.len() > 0 {
                                if let Some(codes) = observation.codes.get_mut(&constell) {
                                    codes.push(observable);
                                } else {
                                    observation.codes.insert(constell, vec![observable]);
                                }
                            }
                        }
                    }
                }
            } else if marker.contains("ANALYSIS CENTER") {
                let (code, agency) = content.split_at(3);
                clocks = clocks.with_agency(ClockAnalysisAgency {
                    code: code.trim().to_string(),
                    name: agency.trim().to_string(),
                });
            } else if marker.contains("# / TYPES OF DATA") {
                let (n, r) = content.split_at(6);
                let n = n.trim();
                let n =
                    u8::from_str_radix(n, 10).or(Err(parse_int_error!("# / TYPES OF DATA", n)))?;

                let mut rem = r.clone();
                for _ in 0..n {
                    let (code, r) = rem.split_at(6);
                    if let Ok(c) = ClockDataType::from_str(code.trim()) {
                        clocks.codes.push(c);
                    }
                    rem = r.clone()
                }
            } else if marker.contains("STATION NAME / NUM") {
                let (name, num) = content.split_at(4);
                clocks = clocks.with_ref_station(clocks::Station {
                    id: num.trim().to_string(),
                    name: name.trim().to_string(),
                });
            } else if marker.contains("STATION CLK REF") {
                clocks = clocks.with_ref_clock(content.trim());
            } else if marker.contains("SIGNAL STRENGHT UNIT") {
                //TODO
            } else if marker.contains("INTERVAL") {
                let intv_str = content.split_at(20).0.trim();
                if let Ok(interval) = f64::from_str(intv_str) {
                    if interval > 0.0 {
                        // INTERVAL = '0' may exist, in case
                        // of Varying TEC map intervals
                        sampling_interval =
                            Some(Duration::from_f64(interval, hifitime::Unit::Second));
                    }
                }
            } else if marker.contains("GLONASS SLOT / FRQ #") {
                //TODO
                // This should be used when dealing with Glonass carriers

                let slots = content.split_at(4).1.trim();
                for i in 0..num_integer::div_ceil(slots.len(), 7) {
                    let svnn = &slots[i * 7..i * 7 + 4];
                    let chx = &slots[i * 7 + 4..std::cmp::min(i * 7 + 4 + 3, slots.len())];
                    if let Ok(svnn) = Sv::from_str(svnn.trim()) {
                        if let Ok(chx) = i8::from_str_radix(chx.trim(), 10) {
                            glo_channels.insert(svnn, chx);
                        }
                    }
                }
            } else if marker.contains("GLONASS COD/PHS/BIS") {
                //TODO
                // This will help RTK solving against GLONASS SV
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
                // GPUT 0.2793967723E-08 0.000000000E+00 147456 1395
                /*
                 * V3 Time System correction description
                 */
                //if let Ok((ts, ts, corr)) = gnss_time::decode_time_system_corr(content) {
                //    time_corrections.insert(ts, (ts, corr));
                //}
            } else if marker.contains("TIME SYSTEM ID") {
                let timescale = content.trim();
                let ts = TimeScale::from_str(timescale)
                    .or(Err(ParsingError::TimescaleParsing(timescale.to_string())))?;
                clocks = clocks.with_timescale(ts);
            } else if marker.contains("DELTA-UTC") {
                //TODO
                //0.931322574615D-09 0.355271367880D-14   233472     1930 DELTA-UTC: A0,A1,T,W
            } else if marker.contains("DESCRIPTION") {
                // IONEX description
                // <o
                //   if "DESCRIPTION" is to be encountered in other RINEX
                //   we can safely test RinexType here because its already been determined
                ionex = ionex.with_description(content.trim())
            } else if marker.contains("OBSERVABLES USED") {
                // IONEX observables
                ionex = ionex.with_observables(content.trim())
            } else if marker.contains("ELEVATION CUTOFF") {
                if let Ok(f) = f32::from_str(content.trim()) {
                    ionex = ionex.with_elevation_cutoff(f);
                }
            } else if marker.contains("BASE RADIUS") {
                if let Ok(f) = f32::from_str(content.trim()) {
                    ionex = ionex.with_base_radius(f);
                }
            } else if marker.contains("MAPPING FUCTION") {
                if let Ok(mf) = ionex::MappingFunction::from_str(content.trim()) {
                    ionex = ionex.with_mapping_function(mf);
                }
            } else if marker.contains("# OF STATIONS") {
                // IONEX
                if let Ok(u) = content.trim().parse::<u32>() {
                    ionex = ionex.with_nb_stations(u)
                }
            } else if marker.contains("# OF SATELLITES") {
                // IONEX
                if let Ok(u) = content.trim().parse::<u32>() {
                    ionex = ionex.with_nb_satellites(u)
                }
            /*
             * Initial TEC map scaling
             */
            } else if marker.contains("EXPONENT") {
                if let Ok(e) = i8::from_str_radix(content.trim(), 10) {
                    ionex = ionex.with_exponent(e);
                }

            /*
             * Ionex Grid Definition
             */
            } else if marker.contains("HGT1 / HGT2 / DHGT") {
                let items: Vec<&str> = content.split_ascii_whitespace().collect();
                if items.len() == 3 {
                    let start = items[0].trim();
                    let start = f64::from_str(start).or(Err(parse_float_error!(
                        "ionex (alt) grid first coordinates",
                        start
                    )))?;

                    let end = items[1].trim();
                    let end = f64::from_str(end).or(Err(parse_float_error!(
                        "ionex (alt) grid last coordinates",
                        end
                    )))?;

                    let spacing = items[2].trim();
                    let spacing = f64::from_str(spacing).or(Err(parse_float_error!(
                        "ionex (alt) grid coordinates spacing",
                        spacing
                    )))?;

                    let grid = match spacing == 0.0 {
                        true => {
                            // special case, 2D fixed altitude
                            ionex::GridLinspace {
                                // avoid verifying the Linspace in this case
                                start,
                                end,
                                spacing: 0.0,
                            }
                        },
                        _ => ionex::GridLinspace::new(start, end, spacing)?,
                    };

                    ionex = ionex.with_altitude_grid(grid);
                } else {
                    return Err(grid_format_error!("HGT1 / HGT2 / DGHT", content));
                }
            } else if marker.contains("LAT1 / LAT2 / DLAT") {
                let items: Vec<&str> = content.split_ascii_whitespace().collect();
                if items.len() == 3 {
                    let start = items[0].trim();
                    let start = f64::from_str(start).or(Err(parse_float_error!(
                        "ionex (lat) grid first coordinates",
                        start
                    )))?;

                    let end = items[1].trim();
                    let end = f64::from_str(end).or(Err(parse_float_error!(
                        "ionex (lat) grid last coordinates",
                        end
                    )))?;

                    let spacing = items[2].trim();
                    let spacing = f64::from_str(spacing).or(Err(parse_float_error!(
                        "ionex (lat) grid coordinates spacing",
                        spacing
                    )))?;

                    ionex =
                        ionex.with_latitude_grid(ionex::GridLinspace::new(start, end, spacing)?);
                } else {
                    return Err(grid_format_error!("LAT1 / LAT2 / DLAT", content));
                }
            } else if marker.contains("LON1 / LON2 / DLON") {
                let items: Vec<&str> = content.split_ascii_whitespace().collect();
                if items.len() == 3 {
                    let start = items[0].trim();
                    let start = f64::from_str(start).or(Err(parse_float_error!(
                        "ionex (lon) grid first coordinates",
                        start
                    )))?;

                    let end = items[1].trim();
                    let end = f64::from_str(end).or(Err(parse_float_error!(
                        "ionex (lon) grid last coordinates",
                        end
                    )))?;

                    let spacing = items[2].trim();
                    let spacing = f64::from_str(spacing).or(Err(parse_float_error!(
                        "ionex (lon) grid coordinates spacing",
                        spacing
                    )))?;

                    ionex =
                        ionex.with_longitude_grid(ionex::GridLinspace::new(start, end, spacing)?);
                } else {
                    return Err(grid_format_error!("LON1 / LON2 / DLON", content));
                }
            } else if marker.contains("PRN / BIAS / RMS") {
                // differential PR code analysis
                //TODO
            }
        }

        Ok(Header {
            version,
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
            glo_channels,
            leap,
            ground_position,
            dcb_compensations,
            pcv_compensations,
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
    pub fn with_general_infos(&self, program: &str, run_by: &str, agency: &str) -> Self {
        let mut s = self.clone();
        s.program = program.to_string();
        s.run_by = run_by.to_string();
        s.agency = agency.to_string();
        s
    }

    /// Adds crinex generation attributes to self,
    /// has no effect if this is not an Observation Data header.
    pub fn with_crinex(&self, c: Crinex) -> Self {
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
    pub fn with_constellation(&self, c: Constellation) -> Self {
        let mut s = self.clone();
        s.constellation = Some(c);
        s
    }

    /// adds comments to Self
    pub fn with_comments(&self, c: Vec<String>) -> Self {
        let mut s = self.clone();
        s.comments = c.clone();
        s
    }

    pub fn with_observation_fields(&self, fields: observation::HeaderFields) -> Self {
        let mut s = self.clone();
        s.obs = Some(fields);
        s
    }

    fn parse_time_of_obs(content: &str) -> Result<Epoch, ParsingError> {
        let (_, rem) = content.split_at(2);
        let (y, rem) = rem.split_at(4);
        let (m, rem) = rem.split_at(6);
        let (d, rem) = rem.split_at(6);
        let (hh, rem) = rem.split_at(6);
        let (mm, rem) = rem.split_at(6);
        let (ss, rem) = rem.split_at(5);
        let (_dot, rem) = rem.split_at(1);
        let (ns, rem) = rem.split_at(8);

        // println!("Y \"{}\" M \"{}\" D \"{}\" HH \"{}\" MM \"{}\" SS \"{}\" NS \"{}\"", y, m, d, hh, mm, ss, ns); // DEBUG
        let y = u32::from_str_radix(y.trim(), 10)
            .map_err(|_| ParsingError::DateTimeParsing(String::from("year"), y.to_string()))?;

        let m = u8::from_str_radix(m.trim(), 10)
            .map_err(|_| ParsingError::DateTimeParsing(String::from("months"), m.to_string()))?;

        let d = u8::from_str_radix(d.trim(), 10)
            .map_err(|_| ParsingError::DateTimeParsing(String::from("days"), d.to_string()))?;

        let hh = u8::from_str_radix(hh.trim(), 10)
            .map_err(|_| ParsingError::DateTimeParsing(String::from("hours"), hh.to_string()))?;

        let mm = u8::from_str_radix(mm.trim(), 10)
            .map_err(|_| ParsingError::DateTimeParsing(String::from("minutes"), mm.to_string()))?;

        let ss = u8::from_str_radix(ss.trim(), 10)
            .map_err(|_| ParsingError::DateTimeParsing(String::from("seconds"), ss.to_string()))?;

        let ns = u32::from_str_radix(ns.trim(), 10)
            .map_err(|_| ParsingError::DateTimeParsing(String::from("nanos"), ns.to_string()))?;

        /* timescale might be missing in OLD RINEX: we handle that externally */
        let mut ts = TimeScale::TAI;

        let rem = rem.trim();
        if rem.len() > 0 {
            // println!("TS \"{}\"", rem); // DBEUG
            ts = TimeScale::from_str(rem.trim()).map_err(|_| {
                ParsingError::DateTimeParsing(String::from("timescale"), rem.to_string())
            })?;
        }

        Ok(Epoch::from_str(&format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:08} {}",
            y, m, d, hh, mm, ss, ns, ts
        ))
        .map_err(|_| ParsingError::DateTimeParsing(String::from("timescale"), rem.to_string()))?)
    }
}

impl std::fmt::Display for Header {
    /// `Header` formatter, mainly for RINEX file production purposes
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // start with CRINEX attributes, if need be
        if let Some(obs) = &self.obs {
            if let Some(crinex) = &obs.crinex {
                write!(f, "{}\n", crinex)?;
            }
        }
        // RINEX VERSION / TYPE
        write!(
            f,
            "{:6}.{:02}           ",
            self.version.major, self.version.minor
        )?;
        match self.rinex_type {
            Type::NavigationData => {
                match self.constellation {
                    Some(Constellation::Glonass) => {
                        // Glonass Special case
                        write!(f, "{:<20}", "G: GLONASS NAV DATA")?;
                        write!(f, "{:<20}", "")?;
                        write!(f, "{}", "RINEX VERSION / TYPE\n")?
                    },
                    Some(c) => {
                        write!(f, "{:<20}", "NAVIGATION DATA")?;
                        let constell = format!("{:x}", c);
                        write!(f, "{:<20}", constell)?;
                        write!(f, "{:<20}", "RINEX VERSION / TYPE\n")?
                    },
                    _ => panic!("constellation must be specified when formatting a NavigationData"),
                }
            },
            Type::ObservationData => match self.constellation {
                Some(c) => {
                    write!(f, "{:<20}", "OBSERVATION DATA")?;
                    let constell = format!("{:x}", c);
                    write!(f, "{:<20}", constell)?;
                    write!(f, "{:<20}", "RINEX VERSION / TYPE\n")?
                },
                _ => panic!("constellation must be specified when formatting ObservationData"),
            },
            Type::MeteoData => {
                write!(f, "{:<20}", "METEOROLOGICAL DATA")?;
                write!(f, "{:<20}", "")?;
                write!(f, "{:<20}", "RINEX VERSION / TYPE\n")?;
            },
            Type::ClockData => {
                write!(f, "{:<20}", "CLOCK DATA")?;
                write!(f, "{:<20}", "")?;
                write!(f, "{:<20}", "RINEX VERSION / TYPE\n")?;
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
        if self.observer.len() + self.agency.len() > 0 {
            write!(f, "{:<20}", self.observer)?;
            write!(f, "{:<40}", self.agency)?;
            write!(f, "OBSERVER / AGENCY\n")?;
        }
        // MARKER NAME
        if self.station.len() > 0 {
            write!(f, "{:<20}", self.station)?;
            write!(f, "{:<40}", " ")?;
            write!(f, "{}", "MARKER NAME\n")?;
        }
        // MARKER NUMBER
        if self.station_id.len() > 0 {
            // has been parsed
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
                write!(f, "{:14.4}", coords.0)?;
                write!(f, "{:14.4}", coords.1)?;
                write!(f, "{:14.4}", coords.2)?;
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
            write!(f, "{:6}", interval.to_seconds())?;
            write!(f, "{:<54}", "")?;
            write!(f, "INTERVAL\n")?
        }
        // List of Observables
        match self.rinex_type {
            Type::ObservationData => {
                if let Some(obs) = &self.obs {
                    if let Some(time_of_first_obs) = obs.time_of_first_obs {
                        //TODO: hifitime does not have a gregorian decomposition method at the moment
                        //let offset = match time_of_first_obs.time_scale {
                        //    TimeScale::GPST => Duration::from_seconds(19.0),
                        //    TimeScale::GST => Duration::from_seconds(35.0),
                        //    TimeScale::BDT => Duration::from_seconds(35.0),
                        //    _ => Duration::default(),
                        //};
                        let (y, m, d, hh, mm, ss, nanos) = (time_of_first_obs).to_gregorian_utc();
                        let mut descriptor = format!(
                            "  {:04}    {:02}    {:02}    {:02}    {:02}   {:02}.{:07}     {:x}",
                            y, m, d, hh, mm, ss, nanos, time_of_first_obs.time_scale
                        );
                        descriptor.push_str(&format!(
                            "{:<width$}",
                            "",
                            width = 60 - descriptor.len()
                        ));
                        descriptor.push_str("TIME OF FIRST OBS\n");
                        write!(f, "{}", descriptor)?;
                    }
                    if let Some(time_of_last_obs) = obs.time_of_last_obs {
                        //TODO: hifitime does not have a gregorian decomposition method at the moment
                        let offset = match time_of_last_obs.time_scale {
                            TimeScale::GPST => Duration::from_seconds(19.0),
                            TimeScale::GST => Duration::from_seconds(35.0),
                            TimeScale::BDT => Duration::from_seconds(35.0),
                            _ => Duration::default(),
                        };
                        let (y, m, d, hh, mm, ss, nanos) =
                            (time_of_last_obs + offset).to_gregorian_utc();
                        let mut descriptor = format!(
                            "  {:04}    {:02}    {:02}  {:02}   {:02}  {:02}.{:08}   {:x}",
                            y, m, d, hh, mm, ss, nanos, time_of_last_obs.time_scale
                        );
                        descriptor.push_str(&format!(
                            "{:<width$}",
                            "",
                            width = 60 - descriptor.len()
                        ));
                        descriptor.push_str("TIME OF LAST OBS\n");
                        write!(f, "{}", descriptor)?;
                    }
                    match self.version.major {
                        1 | 2 => {
                            // old revisions
                            for (_, observables) in obs.codes.iter() {
                                write!(f, "{:6}", observables.len())?;
                                let mut descriptor = String::new();
                                for (i, observable) in observables.iter().enumerate() {
                                    if (i % 9) == 0 && i > 0 {
                                        //ADD LABEL
                                        descriptor.push_str("# / TYPES OF OBSERV\n");
                                        descriptor.push_str(&format!("{:<6}", ""));
                                        //TAB
                                    }
                                    // <!> this will not work if observable
                                    //     does not fit on 2 characters
                                    descriptor.push_str(&format!("    {}", observable));
                                }
                                //ADD BLANK on last line
                                if observables.len() <= 9 {
                                    // fits on one line
                                    descriptor.push_str(&format!(
                                        "{:<width$}",
                                        "",
                                        width = 80 - descriptor.len()
                                    ));
                                } else {
                                    let nb_lines = observables.len() / 9;
                                    let blanking = 80 - (descriptor.len() - 80 * nb_lines); //98 = 80 + # / TYPESOFOBSERV
                                    descriptor.push_str(&format!(
                                        "{:<width$}",
                                        "",
                                        width = blanking
                                    ));
                                }
                                //ADD LABEL
                                descriptor.push_str("# / TYPES OF OBSERV\n");
                                write!(f, "{}", descriptor)?;
                                // NOTE ON THIS BREAK
                                //      header contains obs.codes[] copied for every possible constellation system
                                //      because we have no means to known which ones are to be encountered
                                //      in this great/magnificent RINEX2 format.
                                //      On the other hand, we're expected to only declare a single #/TYPESOFOBSERV label
                                break;
                            }
                        },
                        _ => {
                            // modern revisions
                            for (constell, codes) in obs.codes.iter() {
                                let mut line = format!("{:x<4}", constell);
                                line.push_str(&format!("{:2}", codes.len()));
                                for (i, code) in codes.iter().enumerate() {
                                    if (i + 1) % 14 == 0 {
                                        line.push_str(&format!(
                                            "{:<width$}",
                                            "",
                                            width = 60 - line.len()
                                        ));
                                        line.push_str("SYS / # / OBS TYPES\n");
                                        write!(f, "{}", line)?;
                                        line.clear();
                                        line.push_str(&format!("{:<6}", "")); //TAB
                                    }
                                    line.push_str(&format!(" {}", code))
                                }
                                line.push_str(&format!("{:<width$}", "", width = 60 - line.len()));
                                line.push_str("SYS / # / OBS TYPES\n");
                                write!(f, "{}", line)?
                            }
                        },
                    }
                }
            }, //ObservationData observables description
            Type::MeteoData => {
                if let Some(obs) = &self.meteo {
                    write!(f, "{:6}", obs.codes.len())?;
                    let mut description = String::new();
                    for i in 0..obs.codes.len() {
                        if (i % 9) == 0 && i > 0 {
                            description.push_str("# / TYPES OF OBSERV\n");
                            write!(f, "{}", description)?;
                            description.clear();
                            description.push_str(&format!("{:<6}", "")); //TAB
                        }
                        description.push_str(&format!("    {}", obs.codes[i]));
                    }
                    description.push_str(&format!(
                        "{:<width$}",
                        "",
                        width = 54 - description.len()
                    ));
                    description.push_str("# / TYPES OF OBSERV\n");
                    write!(f, "{}", description)?
                }
            }, //MeteoData observables description
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
            line.push_str(&format!(
                "{:>width$}",
                "LEAP SECONDS\n",
                width = 73 - line.len()
            ));
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
            writeln!(
                f,
                "{:>width$}",
                "# / TYPES OF DATA",
                width = 80 - 6 - 6 * clocks.codes.len() - 2
            )?;

            // possible timescale
            if let Some(ts) = clocks.timescale {
                write!(
                    f,
                    "   {:x}                                                     TIME SYSTEM ID\n",
                    ts
                )?;
            }
            // possible reference agency
            if let Some(agency) = &clocks.agency {
                write!(f, "{:<5} ", agency.code)?;
                write!(f, "{}", agency.name)?;
                writeln!(f, "ANALYSIS CENTER")?;
            }
            // possible reference clock information
        }
        // Custom IONEX fields
        if let Some(ionex) = &self.ionex {
            //TODO:
            //  EPOCH OF FIRST and LAST MAP
            //   with epoch::format(Ionex)
            let _ = writeln!(f, "{:6}           MAP DIMENSION", ionex.map_dimension);
            let h = &ionex.grid.height;
            let _ = writeln!(
                f,
                "{} {}  {}     HGT1 / HGT2 / DHGT",
                h.start, h.end, h.spacing
            );
            let lat = &ionex.grid.latitude;
            let _ = writeln!(
                f,
                "{} {}  {}     LAT1 / LON2 / DLAT",
                lat.start, lat.end, lat.spacing
            );
            let lon = &ionex.grid.longitude;
            let _ = writeln!(
                f,
                "{} {}  {}     LON1 / LON2 / DLON",
                lon.start, lon.end, lon.spacing
            );
            let _ = writeln!(f, "{}         ELEVATION CUTOFF", ionex.elevation_cutoff);
            if let Some(func) = &ionex.mapping {
                let _ = writeln!(f, "{:?}         MAPPING FUNCTION", func);
            } else {
                let _ = writeln!(f, "NONE         MAPPING FUNCTION");
            }
            let _ = writeln!(f, "{}               EXPONENT", ionex.exponent);
            if let Some(desc) = &ionex.description {
                for line in 0..desc.len() / 60 {
                    let max = std::cmp::min((line + 1) * 60, desc.len());
                    let _ = writeln!(f, "{}                COMMENT", &desc[line * 60..max]);
                }
            }
        }
        // END OF HEADER
        writeln!(f, "{:>74}", "END OF HEADER")
    }
}

impl Header {
    /*
     * Macro to be used when marking Self as Merged file
     */
    fn merge_comment(timestamp: Epoch) -> String {
        let (y, m, d, hh, mm, ss, _) = timestamp.to_gregorian_utc();
        format!(
            "rustrnx-{:<11} FILE MERGE          {}{}{} {}{}{} {:x}",
            env!("CARGO_PKG_VERSION"),
            y,
            m,
            d,
            hh,
            mm,
            ss,
            timestamp.time_scale
        )
    }
}

impl Merge for Header {
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

        // sampling interval special case
        match self.sampling_interval {
            None => {
                if rhs.sampling_interval.is_some() {
                    self.sampling_interval = rhs.sampling_interval.clone();
                }
            },
            Some(lhs) => {
                if let Some(rhs) = rhs.sampling_interval {
                    self.sampling_interval = Some(std::cmp::min(lhs, rhs));
                }
            },
        }

        merge::merge_mut_vec(&mut self.comments, &rhs.comments);
        merge::merge_mut_option(&mut self.marker_type, &rhs.marker_type);
        merge::merge_mut_option(&mut self.license, &rhs.license);
        merge::merge_mut_option(&mut self.data_scaling, &rhs.data_scaling);
        merge::merge_mut_option(&mut self.doi, &rhs.doi);
        merge::merge_mut_option(&mut self.leap, &rhs.leap);
        merge::merge_mut_option(&mut self.gps_utc_delta, &rhs.gps_utc_delta);
        merge::merge_mut_option(&mut self.rcvr, &rhs.rcvr);
        merge::merge_mut_option(&mut self.rcvr_antenna, &rhs.rcvr_antenna);
        merge::merge_mut_option(&mut self.sv_antenna, &rhs.sv_antenna);
        merge::merge_mut_option(&mut self.ground_position, &rhs.ground_position);
        merge::merge_mut_option(&mut self.wavelengths, &rhs.wavelengths);
        merge::merge_mut_option(&mut self.gps_utc_delta, &rhs.gps_utc_delta);

        // DCBS compensation is preserved, only if both A&B both have it
        if self.dcb_compensations.len() == 0 || rhs.dcb_compensations.len() == 0 {
            self.dcb_compensations.clear(); // drop everything
        } else {
            let rhs_constellations: Vec<_> = rhs
                .dcb_compensations
                .iter()
                .map(|dcb| dcb.constellation.clone())
                .collect();
            self.dcb_compensations
                .iter_mut()
                .filter(|dcb| rhs_constellations.contains(&dcb.constellation))
                .count();
        }

        // PCV compensation : same logic
        // only preserve compensations present in both A & B
        if self.pcv_compensations.len() == 0 || rhs.pcv_compensations.len() == 0 {
            self.pcv_compensations.clear(); // drop everything
        } else {
            let rhs_constellations: Vec<_> = rhs
                .pcv_compensations
                .iter()
                .map(|pcv| pcv.constellation.clone())
                .collect();
            self.dcb_compensations
                .iter_mut()
                .filter(|pcv| rhs_constellations.contains(&pcv.constellation))
                .count();
        }

        //TODO :
        //merge::merge_mut(&mut self.glo_channels, &rhs.glo_channels);

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
                // TODO: manage that
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

                //TODO: this is not enough, need to take into account and rescale..
                lhs.exponent = std::cmp::min(lhs.exponent, rhs.exponent);

                merge::merge_mut_option(&mut lhs.description, &rhs.description);
                merge::merge_mut_option(&mut lhs.mapping, &rhs.mapping);
                if lhs.elevation_cutoff == 0.0 {
                    // means "unknown"
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
        // add special comment
        let now = Epoch::now()?;
        let merge_comment = Self::merge_comment(now);
        self.comments.push(merge_comment);
        Ok(())
    }
}

#[cfg(feature = "qc")]
use horrorshow::{helper::doctype, RenderBox};

#[cfg(feature = "qc")]
use rinex_qc_traits::HtmlReport;

#[cfg(feature = "qc")]
impl HtmlReport for Header {
    fn to_html(&self) -> String {
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        meta(content="text/html", charset="utf-8");
                        meta(name="viewport", content="width=device-width, initial-scale=1");
                        link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
                        title {
                            : "RINEX Header analysis"
                        }
                    }
                    body {
                        : self.to_inline_html()
                    }
                }
            }
        )
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            tr {
                th {
                    : "Antenna"
                }
                @ if let Some(antenna) = &self.rcvr_antenna {
                    td {
                        : antenna.to_inline_html()
                    }
                } else {
                    td {
                        : "No information"
                    }
                }
            }
            tr {
                th {
                    : "Receiver"
                }
                @ if let Some(rcvr) = &self.rcvr {
                    td {
                        : rcvr.to_inline_html()
                    }
                } else {
                    td {
                        : "No information"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::parse_formatted_month;
    #[test]
    fn formatted_month_parser() {
        for (desc, expected) in vec![("Jan", 1), ("Feb", 2), ("Mar", 3), ("Nov", 11), ("Dec", 12)] {
            let month = parse_formatted_month(desc);
            assert!(month.is_ok(), "failed to parse month from \"{}\"", desc);
            let month = month.unwrap();
            assert_eq!(
                month, expected,
                "failed to parse correct month number from \"{}\"",
                desc
            );
        }
    }
}
