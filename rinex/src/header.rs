//! Describes a `RINEX` file header.
use crate::{
    antex::{HeaderFields as AntexHeader, Pcv},
    clock::{ClockProfileType, HeaderFields as ClockHeader, WorkClock},
    doris::{HeaderFields as DorisHeader, Station as DorisStation},
    epoch::parse_ionex_utc as parse_ionex_utc_epoch,
    fmt_comment, fmt_rinex,
    ground_position::GroundPosition,
    hardware::{Antenna, Receiver, SvAntenna},
    hatanaka::CRINEX,
    ionex::{
        HeaderFields as IonexHeaderFields, MappingFunction as IonexMappingFunction,
        RefSystem as IonexRefSystem,
    },
    leap::Leap,
    linspace::Linspace,
    marker::{GeodeticMarker, MarkerType},
    meteo::{sensor::Sensor as MeteoSensor, HeaderFields as MeteoHeader},
    navigation::{IonMessage, KbModel},
    observable::Observable,
    observation::HeaderFields as ObservationHeader,
    prelude::{Constellation, Duration, Epoch, ParsingError, TimeScale, COSPAR, DOMES, SV},
    reader::BufferedReader,
    types::Type,
    version::Version,
};

use std::{
    collections::HashMap,
    io::{BufRead, Read},
    str::FromStr,
};

use hifitime::Unit;

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "qc")]
use maud::{html, Markup, Render};

#[cfg(feature = "processing")]
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

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
#[cfg_attr(feature = "serde", derive(Serialize))]
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
    /// optionnal station/marker/agency URL
    pub station_url: String,
    /// name of observer
    pub observer: String,
    /// name of production agency
    pub agency: String,
    /// optionnal [GeodeticMarker]
    pub geodetic_marker: Option<GeodeticMarker>,
    /// Glonass FDMA channels
    pub glo_channels: HashMap<SV, i8>,
    /// Optional COSPAR number (launch information)
    pub cospar: Option<COSPAR>,
    /// optionnal leap seconds infos
    pub leap: Option<Leap>,
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
    /// Optionnal Receiver information
    #[cfg_attr(feature = "serde", serde(default))]
    pub rcvr: Option<Receiver>,
    /// Optionnal Receiver Antenna information
    #[cfg_attr(feature = "serde", serde(default))]
    pub rcvr_antenna: Option<Antenna>,
    /// Optionnal Vehicle Antenna information,
    /// attached to a specifid SV, only exists in ANTEX records
    #[cfg_attr(feature = "serde", serde(default))]
    pub sv_antenna: Option<SvAntenna>,
    /// Possible Ionospheric Delay correction model, described in
    /// header section of old RINEX files (<V4).
    pub ionod_corrections: HashMap<Constellation, IonMessage>,
    /// Possible DCBs compensation information
    pub dcb_compensations: Vec<DcbCompensation>,
    /// Possible PCVs compensation information
    pub pcv_compensations: Vec<PcvCompensation>,
    /// Observation RINEX specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub obs: Option<ObservationHeader>,
    /// Meteo RINEX specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub meteo: Option<MeteoHeader>,
    /// High Precision Clock RINEX specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub clock: Option<ClockHeader>,
    /// ANTEX specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub antex: Option<AntexHeader>,
    /// IONEX specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub ionex: Option<IonexHeaderFields>,
    /// DORIS RINEX specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub doris: Option<DorisHeader>,
}

impl Header {
    /// Builds [Header] from [Read]able interface
    pub fn new<const M: usize, R: Read>(
        reader: &mut BufferedReader<M, R>,
    ) -> Result<Header, ParsingError> {
        let mut rinex_type = Type::default();
        let mut constellation: Option<Constellation> = None;
        let mut version = Version::default();
        let mut comments: Vec<String> = Vec::new();
        let mut program = String::new();
        let mut run_by = String::new();
        let mut date = String::new();
        let mut observer = String::new();
        let mut agency = String::new();
        let mut license: Option<String> = None;
        let mut doi: Option<String> = None;
        let mut station_url = String::new();
        let mut geodetic_marker = Option::<GeodeticMarker>::None;
        let mut cospar = Option::<COSPAR>::None;
        let mut glo_channels: HashMap<SV, i8> = HashMap::new();
        let mut rcvr: Option<Receiver> = None;
        let mut rcvr_antenna: Option<Antenna> = None;
        let mut sv_antenna: Option<SvAntenna> = None;
        let mut leap: Option<Leap> = None;
        let mut sampling_interval: Option<Duration> = None;
        let mut ground_position: Option<GroundPosition> = None;
        let mut dcb_compensations: Vec<DcbCompensation> = Vec::new();
        let mut ionod_corrections = HashMap::<Constellation, IonMessage>::with_capacity(4);
        let mut pcv_compensations: Vec<PcvCompensation> = Vec::new();

        // RINEX specific fields
        let mut current_constell: Option<Constellation> = None;
        let mut observation = ObservationHeader::default();
        let mut meteo = MeteoHeader::default();
        let mut clock = ClockHeader::default();
        let mut antex = AntexHeader::default();
        let mut ionex = IonexHeaderFields::default();
        let mut doris = DorisHeader::default();

        for l in reader.lines() {
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
            // COMMENTS are stored: "as is"
            ///////////////////////////////
            if marker.trim().eq("COMMENT") {
                // --> storing might be useful
                comments.push(content.trim().to_string());
                continue;

            ///////////////////////////////////////////////////////
            // Handled elsewe: CRINEX specs
            //     handled inside the smart I/O READER
            //     we still have to grab what was idenfied though
            //     and we do this at the end of the Header section
            ///////////////////////////////////////////////////////
            } else if marker.contains("CRINEX VERS") {
            } else if marker.contains("CRINEX PROG / DATE") {

                ///////////////////////////////////////////////////////
                // Unhandled cases: TODO
                ///////////////////////////////////////////////////////
            } else if marker.contains("ANTENNA: B.SIGHT XYZ") {
            } else if marker.contains("ANTENNA: ZERODIR XYZ") {
            } else if marker.contains("ANTENNA: PHASECENTER") {
            } else if marker.contains("CENTER OF MASS: XYZ") {
            } else if marker.contains("PRN / BIAS / RMS") {
            } else if marker.contains("DELTA-UTC") {
            } else if marker.contains("TIME REF STATION") {

                ///////////////////////////////////////////////////////
                // Handled cases
                ///////////////////////////////////////////////////////
            } else if marker.contains("ANTEX VERSION / SYST") {
                let (vers, system) = content.split_at(8);
                let vers = vers.trim();
                version = Version::from_str(vers).or(Err(ParsingError::AntexVersion))?;

                if let Ok(constell) = Constellation::from_str(system.trim()) {
                    constellation = Some(constell)
                }
                rinex_type = Type::AntennaData;
            } else if marker.contains("PCV TYPE / REFANT") {
                let (pcv_str, rem) = content.split_at(20);
                let (rel_type, rem) = rem.split_at(20);
                let (ref_sn, _) = rem.split_at(20);
                if let Ok(mut pcv) = Pcv::from_str(pcv_str.trim()) {
                    if pcv.is_relative() {
                        // try to parse "Relative Type"
                        if !rel_type.trim().is_empty() {
                            pcv = pcv.with_relative_type(rel_type.trim());
                        }
                    }
                    antex = antex.with_pcv_type(pcv);
                }
                if !ref_sn.trim().is_empty() {
                    antex = antex.with_reference_antenna_sn(ref_sn.trim());
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
                    if let Ok(sv) = SV::from_str(svnn.trim()) {
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
                version = Version::from_str(vers_str).or(Err(ParsingError::IonexVersion))?;

                rinex_type = Type::from_str(type_str.trim())?;
                let ref_system = IonexRefSystem::from_str(system_str.trim())?;
                ionex = ionex.with_reference_system(ref_system);

            ///////////////////////////////////////
            // ==> from now on
            // RINEX standard / shared attributes
            ///////////////////////////////////////
            } else if marker.contains("RINEX VERSION / TYPE") {
                let (vers, rem) = line.split_at(20);
                let (type_str, rem) = rem.split_at(20);
                let (constell_str, _) = rem.split_at(20);

                let type_str = type_str.trim();
                let constell_str = constell_str.trim();

                // File type identification
                if type_str == "O" && constell_str == "D" {
                    rinex_type = Type::DORIS;
                } else {
                    rinex_type = Type::from_str(type_str)?;
                }

                // Determine (file) Constellation
                //  1. NAV SPECIAL CASE
                //  2. OTHER
                match rinex_type {
                    Type::NavigationData => {
                        if type_str.contains("GLONASS") {
                            // old GLONASS NAV : no constellation field
                            constellation = Some(Constellation::Glonass);
                        } else if type_str.contains("GPS NAV DATA") {
                            constellation = Some(Constellation::GPS);
                        } else if type_str.contains("IRNSS NAV DATA") {
                            constellation = Some(Constellation::IRNSS);
                        } else if type_str.contains("GNSS NAV DATA") {
                            constellation = Some(Constellation::Mixed);
                        } else if type_str.eq("NAVIGATION DATA") {
                            if constell_str.is_empty() {
                                // old GPS NAVIGATION DATA
                                constellation = Some(Constellation::GPS);
                            } else {
                                // Modern NAVIGATION DATA
                                if let Ok(c) = Constellation::from_str(constell_str) {
                                    constellation = Some(c);
                                }
                            }
                        }
                    },
                    Type::MeteoData | Type::DORIS => {
                        // no constellation associated to them
                    },
                    _ => {
                        // any other
                        // regular files
                        if let Ok(c) = Constellation::from_str(constell_str) {
                            constellation = Some(c);
                        }
                    },
                }
                /*
                 * Parse version descriptor
                 */
                let vers = vers.trim();
                version = Version::from_str(vers).or(Err(ParsingError::VersionParsing))?;

                if !version.is_supported() {
                    return Err(ParsingError::NonSupportedVersion);
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
                let name = content.split_at(20).0.trim();
                geodetic_marker = Some(GeodeticMarker::default().with_name(name));
            } else if marker.contains("MARKER NUMBER") {
                let number = content.split_at(20).0.trim();
                if let Some(ref mut marker) = geodetic_marker {
                    *marker = marker.with_number(number);
                }
            } else if marker.contains("MARKER TYPE") {
                let code = content.split_at(20).0.trim();
                if let Ok(mtype) = MarkerType::from_str(code) {
                    if let Some(ref mut marker) = geodetic_marker {
                        marker.marker_type = Some(mtype);
                    }
                }
            } else if marker.contains("OBSERVER / AGENCY") {
                let (obs, ag) = content.split_at(20);
                observer = obs.trim().to_string();
                agency = ag.trim().to_string();
            } else if marker.contains("REC # / TYPE / VERS") {
                if let Ok(receiver) = Receiver::from_str(content) {
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
                    constellation: gnss,
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
                    constellation: gnss,
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
                // TODO:
                //   This will not work in case several observables
                //   are declaredn which will required to analyze more than 1 line
                let (gnss, rem) = content.split_at(2);
                let gnss = gnss.trim();

                /*
                 * DORIS measurement special case, otherwise, standard OBS_RINEX
                 */
                let constell = if gnss.eq("D") {
                    Constellation::Mixed // scaling applies to all measurements
                } else {
                    Constellation::from_str(gnss)?
                };

                // Parse scaling factor
                let (factor, rem) = rem.split_at(6);
                let factor = factor.trim();
                let scaling = factor
                    .parse::<u16>()
                    .or(Err(ParsingError::SystemScalingFactor))?;

                // parse end of line
                let (_num, rem) = rem.split_at(3);
                for observable_str in rem.split_ascii_whitespace() {
                    let observable = Observable::from_str(observable_str)?;

                    // latch scaling value
                    if rinex_type == Type::DORIS {
                        doris.with_scaling(observable, scaling);
                    } else {
                        observation.with_scaling(constell, observable, scaling);
                    }
                }
            } else if marker.contains("SENSOR MOD/TYPE/ACC") {
                if let Ok(sensor) = MeteoSensor::from_str(content) {
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
                let x = f64::from_str(x).or(Err(ParsingError::SensorCoordinates))?;

                let y = y.trim();
                let y = f64::from_str(y).or(Err(ParsingError::SensorCoordinates))?;

                let z = z.trim();
                let z = f64::from_str(z).or(Err(ParsingError::SensorCoordinates))?;

                let h = h.trim();
                let h = f64::from_str(h).or(Err(ParsingError::SensorCoordinates))?;

                for sensor in meteo.sensors.iter_mut() {
                    if sensor.observable == observable {
                        *sensor = sensor.with_position(GroundPosition::from_ecef_wgs84((x, y, z)));
                        *sensor = sensor.with_height(h);
                    }
                }
            } else if marker.contains("LEAP SECOND") {
                let leap_str = content.split_at(40).0.trim();
                let parsed = Leap::from_str(leap_str)?;
                leap = Some(parsed.clone());
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
                let x = f64::from_str(x).or(Err(ParsingError::Coordinates))?;

                let y = items[1].trim();
                let y = f64::from_str(y).or(Err(ParsingError::Coordinates))?;

                let z = items[2].trim();
                let z = f64::from_str(z).or(Err(ParsingError::Coordinates))?;

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
                let x = f64::from_str(x).or(Err(ParsingError::AntennaCoordinates))?;

                let y = items[1].trim();
                let y = f64::from_str(y).or(Err(ParsingError::AntennaCoordinates))?;

                let z = items[2].trim();
                let z = f64::from_str(z).or(Err(ParsingError::AntennaCoordinates))?;

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
            } else if marker.contains("RCV CLOCK OFFS APPL") {
                let value = content.split_at(20).0.trim();
                let n =
                    i32::from_str_radix(value, 10).or(Err(ParsingError::RcvClockOffsApplied))?;

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
                        time_of_first_obs.time_scale =
                            c.timescale().ok_or(ParsingError::NoTimescaleDefinition)?;
                    },
                }
                if rinex_type == Type::DORIS {
                    doris.time_of_first_obs = Some(time_of_first_obs);
                } else {
                    observation = observation.with_timeof_first_obs(time_of_first_obs);
                }
            } else if marker.contains("TIME OF LAST OBS") {
                let mut time_of_last_obs = Self::parse_time_of_obs(content)?;
                match constellation {
                    Some(Constellation::Mixed) | None => {},
                    Some(c) => {
                        // in case of OLD RINEX : fixed constellation
                        //  use that information, as it may be omitted in the TIME OF OBS header
                        time_of_last_obs.time_scale =
                            c.timescale().ok_or(ParsingError::NoTimescaleDefinition)?;
                    },
                }

                if rinex_type == Type::DORIS {
                    doris.time_of_last_obs = Some(time_of_last_obs);
                } else {
                    observation = observation.with_timeof_last_obs(time_of_last_obs);
                }
            } else if marker.contains("TYPES OF OBS") {
                // these observations can serve both Observation & Meteo RINEX
                Self::parse_v2_observables(content, constellation, &mut meteo, &mut observation);
            } else if marker.contains("SYS / # / OBS TYPES") {
                match rinex_type {
                    Type::ObservationData => {
                        Self::parse_v3_observables(
                            content,
                            &mut current_constell,
                            &mut observation,
                        );
                    },
                    Type::DORIS => {
                        /* in DORIS RINEX, observations are not tied to a particular constellation */
                        Self::parse_doris_observables(content, &mut doris);
                    },
                    _ => {},
                }
            } else if marker.contains("ANALYSIS CENTER") {
                let (code, agency) = content.split_at(3);
                clock = clock.igs(code.trim());
                clock = clock.full_name(agency.trim());
            } else if marker.contains("ANALYSIS CLK REF") {
                let ck = WorkClock::parse(version, content);
                clock = clock.work_clock(ck);
            } else if marker.contains("# / TYPES OF DATA") {
                let (n, r) = content.split_at(6);
                let n = n.trim();
                let n = n.parse::<u8>().or(Err(ParsingError::ClockTypeofData))?;

                let mut rem = r;
                for _ in 0..n {
                    let (code, r) = rem.split_at(6);
                    if let Ok(c) = ClockProfileType::from_str(code.trim()) {
                        clock.codes.push(c);
                    }
                    rem = r;
                }
            } else if marker.contains("STATION NAME / NUM") {
                let (name, domes) = content.split_at(4);
                clock = clock.site(name.trim());
                if let Ok(domes) = DOMES::from_str(domes.trim()) {
                    clock = clock.domes(domes);
                }
            } else if marker.contains("STATION CLK REF") {
                clock = clock.refclock(content.trim());
            } else if marker.contains("SIGNAL STRENGHT UNIT") {
                //TODO
            } else if marker.contains("INTERVAL") {
                let intv_str = content.split_at(20).0.trim();
                if let Ok(interval) = f64::from_str(intv_str) {
                    if interval > 0.0 {
                        // INTERVAL = '0' may exist, in case
                        // of Varying TEC map intervals
                        sampling_interval = Some(Duration::from_seconds(interval));
                    }
                }
            } else if marker.contains("COSPAR NUMBER") {
                cospar = Some(COSPAR::from_str(content.trim())?);
            } else if marker.contains("GLONASS SLOT / FRQ #") {
                //TODO
                // This should be used when dealing with Glonass carriers

                let slots = content.split_at(4).1.trim();
                for i in 0..num_integer::div_ceil(slots.len(), 7) {
                    let svnn = &slots[i * 7..i * 7 + 4];
                    let chx = &slots[i * 7 + 4..std::cmp::min(i * 7 + 4 + 3, slots.len())];
                    if let Ok(svnn) = SV::from_str(svnn.trim()) {
                        if let Ok(chx) = chx.trim().parse::<i8>() {
                            glo_channels.insert(svnn, chx);
                        }
                    }
                }
            } else if marker.contains("GLONASS COD/PHS/BIS") {
                //TODO
                // This will help RTK solving against GLONASS SV
            } else if marker.contains("ION ALPHA") {
                // RINEX v2 Ionospheric correction. We tolerate BETA/ALPHA order mixup, as per
                // RINEX v2 standards [https://files.igs.org/pub/data/format/rinex211.txt] paragraph 5.2.
                match IonMessage::from_rinex2_header(content, marker) {
                    Ok(IonMessage::KlobucharModel(KbModel {
                        alpha,
                        beta,
                        region,
                    })) => {
                        // Support GPS|GLO|BDS|GAL|QZSS|SBAS|IRNSS
                        for c in [
                            Constellation::GPS,
                            Constellation::Glonass,
                            Constellation::BeiDou,
                            Constellation::Galileo,
                            Constellation::IRNSS,
                            Constellation::QZSS,
                            Constellation::SBAS,
                        ] {
                            if let Some(correction) = ionod_corrections.get_mut(&c) {
                                // Only Klobuchar models in RINEX2
                                let kb_model = correction.as_klobuchar_mut().unwrap();
                                kb_model.alpha = alpha;
                                kb_model.region = region;
                            } else {
                                ionod_corrections.insert(
                                    c,
                                    IonMessage::KlobucharModel(KbModel {
                                        alpha,
                                        beta,
                                        region,
                                    }),
                                );
                            }
                        }
                    },
                    _ => {},
                }
            } else if marker.contains("ION BETA") {
                // RINEX v2 Ionospheric correction. We are flexible in their order of appearance,
                // RINEX v2 standards do NOT guarantee that (header fields are free order).
                // [https://files.igs.org/pub/data/format/rinex211.txt] paragraph 5.2.
                match IonMessage::from_rinex2_header(content, marker) {
                    Ok(IonMessage::KlobucharModel(KbModel {
                        alpha,
                        beta,
                        region,
                    })) => {
                        // Support GPS|GLO|BDS|GAL|QZSS|SBAS|IRNSS
                        for c in [
                            Constellation::GPS,
                            Constellation::Glonass,
                            Constellation::BeiDou,
                            Constellation::Galileo,
                            Constellation::IRNSS,
                            Constellation::QZSS,
                            Constellation::SBAS,
                        ] {
                            if let Some(correction) = ionod_corrections.get_mut(&c) {
                                // Only Klobuchar models in RINEX2
                                let kb_model = correction.as_klobuchar_mut().unwrap();
                                kb_model.beta = beta;
                            } else {
                                ionod_corrections.insert(
                                    c,
                                    IonMessage::KlobucharModel(KbModel {
                                        alpha,
                                        beta,
                                        region,
                                    }),
                                );
                            }
                        }
                    },
                    _ => {},
                }
            } else if marker.contains("IONOSPHERIC CORR") {
                /*
                 * RINEX3 IONOSPHERIC CORRECTION
                 * We support both model in all RINEX2|RINEX3 constellations.
                 * RINEX4 replaces that with actual file content (body) for improved correction accuracy.
                 * The description requires 2 lines when dealing with KB model and we tolerate order mixup.
                 */
                let model_id = content.split_at(5).0;
                if model_id.len() < 3 {
                    /* BAD RINEX */
                    continue;
                }
                let constell_id = &model_id[..3];
                let constell = match constell_id {
                    "GPS" => Constellation::GPS,
                    "GAL" => Constellation::Galileo,
                    "BDS" => Constellation::BeiDou,
                    "QZS" => Constellation::QZSS,
                    "IRN" => Constellation::IRNSS,
                    "GLO" => Constellation::Glonass,
                    _ => continue,
                };
                match IonMessage::from_rinex3_header(content) {
                    Ok(IonMessage::KlobucharModel(KbModel {
                        alpha,
                        beta,
                        region,
                    })) => {
                        // KB requires two lines
                        if let Some(ionod_model) = ionod_corrections.get_mut(&constell) {
                            let kb_model = ionod_model.as_klobuchar_mut().unwrap();
                            if model_id.ends_with('A') {
                                kb_model.alpha = alpha;
                                kb_model.region = region;
                            } else {
                                kb_model.beta = beta;
                            }
                        } else {
                            // latch new model
                            ionod_corrections.insert(
                                constell,
                                IonMessage::KlobucharModel(KbModel {
                                    alpha,
                                    beta,
                                    region,
                                }),
                            );
                        }
                    },
                    Ok(ion) => {
                        ionod_corrections.insert(constell, ion);
                    },
                    _ => {},
                }
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
                let ts = TimeScale::from_str(timescale)?;
                clock = clock.timescale(ts);
            } else if marker.contains("DESCRIPTION") {
                // IONEX description
                // <o
                //   if "DESCRIPTION" is to be encountered in other RINEX
                //   we can safely test RinexType here because its already been determined
                ionex = ionex.with_description(content.trim());
            } else if marker.contains("EPOCH OF FIRST MAP") {
                if let Ok(epoch) = parse_ionex_utc_epoch(content.trim()) {
                    ionex = ionex.with_epoch_of_first_map(epoch);
                }
            } else if marker.contains("EPOCH OF LAST MAP") {
                if let Ok(epoch) = parse_ionex_utc_epoch(content.trim()) {
                    ionex = ionex.with_epoch_of_last_map(epoch);
                }
            } else if marker.contains("OBSERVABLES USED") {
                // IONEX observables
                ionex = ionex.with_observables(content.trim());
            } else if marker.contains("ELEVATION CUTOFF") {
                if let Ok(f) = f32::from_str(content.trim()) {
                    ionex = ionex.with_elevation_cutoff(f);
                }
            } else if marker.contains("BASE RADIUS") {
                if let Ok(f) = f32::from_str(content.trim()) {
                    ionex = ionex.with_base_radius(f);
                }
            } else if marker.contains("MAPPING FUCTION") {
                let mapf = IonexMappingFunction::from_str(content.trim())?;
                ionex = ionex.with_mapping_function(mapf);
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
                if let Ok(e) = content.trim().parse::<i8>() {
                    ionex = ionex.with_exponent(e);
                }

            /*
             * Ionex Grid Definition
             */
            } else if marker.contains("HGT1 / HGT2 / DHGT") {
                let grid = Self::parse_grid(content)?;
                ionex = ionex.with_altitude_grid(grid);
            } else if marker.contains("LAT1 / LAT2 / DLAT") {
                let grid = Self::parse_grid(content)?;
                ionex = ionex.with_latitude_grid(grid);
            } else if marker.contains("LON1 / LON2 / DLON") {
                let grid = Self::parse_grid(content)?;
                ionex = ionex.with_longitude_grid(grid);
            } else if marker.contains("L2 / L1 DATE OFFSET") {
                // DORIS special case
                let content = content[1..].trim();
                let l2l1_date_offset = content
                    .parse::<f64>()
                    .or(Err(ParsingError::DorisL1L2DateOffset))?;

                doris.l2_l1_date_offset = Duration::from_microseconds(l2l1_date_offset);
            } else if marker.contains("STATION REFERENCE") {
                // DORIS special case
                let station = DorisStation::from_str(content.trim())?;
                doris.stations.push(station);
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
            geodetic_marker,
            agency,
            observer,
            license,
            doi,
            station_url,
            rcvr,
            cospar,
            glo_channels,
            leap,
            ground_position,
            ionod_corrections,
            dcb_compensations,
            pcv_compensations,
            wavelengths: None,
            gps_utc_delta: None,
            sampling_interval,
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
            clock: {
                if rinex_type == Type::ClockData {
                    Some(clock)
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
            doris: {
                if rinex_type == Type::DORIS {
                    Some(doris)
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
            .with_crinex(CRINEX::default())
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
    pub fn with_crinex(&self, c: CRINEX) -> Self {
        let mut s = self.clone();
        if let Some(ref mut obs) = s.obs {
            obs.crinex = Some(c)
        }
        s
    }

    /// Adds receiver information to self
    pub fn with_receiver(&self, r: Receiver) -> Self {
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

    pub fn with_observation_fields(&self, fields: ObservationHeader) -> Self {
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
        let y = y
            .trim()
            .parse::<u32>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let m = m
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let d = d
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let hh = hh
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let mm = mm
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let ss = ss
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let ns = ns
            .trim()
            .parse::<u32>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        /*
         * We set TAI as "default" Timescale.
         * Timescale might be omitted in Old RINEX formats,
         * In this case, we exit with "TAI" and handle that externally.
         */
        let mut ts = TimeScale::TAI;
        let rem = rem.trim();

        /*
         * Handles DORIS measurement special case,
         * offset from TAI, that we will convert back to TAI later
         */
        if !rem.is_empty() && rem != "DOR" {
            ts = TimeScale::from_str(rem.trim())?;
        }

        Epoch::from_str(&format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:08} {}",
            y, m, d, hh, mm, ss, ns, ts
        ))
        .map_err(|_| ParsingError::DatetimeParsing)
    }

    /*
     * Format VERSION/TYPE field
     */
    pub(crate) fn fmt_rinex_version_type(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let major = self.version.major;
        let minor = self.version.minor;
        match self.rinex_type {
            Type::NavigationData => match self.constellation {
                Some(Constellation::Glonass) => {
                    writeln!(
                        f,
                        "{}",
                        fmt_rinex(
                            &format!("{:6}.{:02}           G: GLONASS NAV DATA", major, minor),
                            "RINEX VERSION / TYPE"
                        )
                    )
                },
                Some(c) => {
                    writeln!(
                        f,
                        "{}",
                        fmt_rinex(
                            &format!(
                                "{:6}.{:02}           NAVIGATION DATA     {:X<20}",
                                major, minor, c
                            ),
                            "RINEX VERSION / TYPE"
                        )
                    )
                },
                _ => panic!("constellation must be specified when formatting a NavigationData"),
            },
            Type::ObservationData => match self.constellation {
                Some(c) => {
                    writeln!(
                        f,
                        "{}",
                        fmt_rinex(
                            &format!(
                                "{:6}.{:02}           OBSERVATION DATA    {:x<20}",
                                major, minor, c
                            ),
                            "RINEX VERSION / TYPE"
                        )
                    )
                },
                _ => panic!("constellation must be specified when formatting ObservationData"),
            },
            Type::MeteoData => {
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(
                        &format!("{:6}.{:02}           METEOROLOGICAL DATA", major, minor),
                        "RINEX VERSION / TYPE"
                    )
                )
            },
            Type::ClockData => {
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(
                        &format!("{:6}.{:02}           CLOCK DATA", major, minor),
                        "RINEX VERSION / TYPE"
                    )
                )
            },
            Type::DORIS => todo!("doris formatting"),
            Type::AntennaData => todo!("antex formatting"),
            Type::IonosphereMaps => todo!("ionex formatting"),
        }
    }
    /*
     * Format rinex type dependent stuff
     */
    pub(crate) fn fmt_rinex_dependent(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.rinex_type {
            Type::ObservationData => self.fmt_observation_rinex(f),
            Type::MeteoData => self.fmt_meteo_rinex(f),
            Type::NavigationData => Ok(()),
            Type::ClockData => self.fmt_clock_rinex(f),
            Type::IonosphereMaps => self.fmt_ionex(f),
            Type::AntennaData => Ok(()), // FIXME
            Type::DORIS => Ok(()),       // FIXME
        }
    }
    /*
     * Clock Data fields formatting
     */
    fn fmt_clock_rinex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(clock) = &self.clock {
            // Types of data: observables equivalent
            let mut descriptor = String::new();
            descriptor.push_str(&format!("{:6}", clock.codes.len()));
            for (i, observable) in clock.codes.iter().enumerate() {
                if (i % 9) == 0 && i > 0 {
                    descriptor.push_str("      "); // TAB
                }
                descriptor.push_str(&format!("{:6}", observable));
            }
            writeln!(f, "{}", fmt_rinex(&descriptor, "# / TYPES OF DATA"))?;

            // possible timescale
            if let Some(ts) = clock.timescale {
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(&format!("   {:x}", ts), "TIME SYSTEM ID")
                )?;
            }
            // TODO: missing fields
            //if let Some(agency) = &clock.agency {
            //    writeln!(
            //        f,
            //        "{}",
            //        fmt_rinex(
            //            &format!("{:<5} {}", agency.code, agency.name),
            //            "ANALYSIS CENTER"
            //        )
            //    )?;
            //}
        }
        Ok(())
    }
    /*
     * IONEX fields formatting
     */
    fn fmt_ionex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ionex) = &self.ionex {
            writeln!(
                f,
                "{}",
                fmt_rinex(&format!("{:6}", ionex.map_dimension), "MAP DIMENSION")
            )?;
            // h grid
            let (start, end, spacing) = (
                ionex.grid.height.start,
                ionex.grid.height.end,
                ionex.grid.height.spacing,
            );
            writeln!(
                f,
                "{}",
                fmt_rinex(
                    &format!("{} {} {}", start, end, spacing),
                    "HGT1 / HGT2 / DHGT"
                )
            )?;
            // lat grid
            let (start, end, spacing) = (
                ionex.grid.latitude.start,
                ionex.grid.latitude.end,
                ionex.grid.latitude.spacing,
            );
            writeln!(
                f,
                "{}",
                fmt_rinex(
                    &format!("{} {} {}", start, end, spacing),
                    "LAT1 / LAT2 / DLAT"
                )
            )?;
            // lon grid
            let (start, end, spacing) = (
                ionex.grid.longitude.start,
                ionex.grid.longitude.end,
                ionex.grid.longitude.spacing,
            );
            writeln!(
                f,
                "{}",
                fmt_rinex(
                    &format!("{} {} {}", start, end, spacing),
                    "LON1 / LON2 / DLON"
                )
            )?;
            // elevation cutoff
            writeln!(
                f,
                "{}",
                fmt_rinex(&format!("{}", ionex.elevation_cutoff), "ELEVATION CUTOFF")
            )?;
            // mapping func
            if let Some(func) = &ionex.mapping {
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(&format!("{:?}", func), "MAPPING FUNCTION")
                )?;
            } else {
                writeln!(f, "{}", fmt_rinex("NONE", "MAPPING FUNCTION"))?;
            }
            // time of first map
            writeln!(f, "{}", fmt_rinex("TODO", "EPOCH OF FIRST MAP"))?;
            // time of last map
            writeln!(f, "{}", fmt_rinex("TODO", "EPOCH OF LAST MAP"))?;
        }
        Ok(())
    }
    /*
     * Meteo Data fields formatting
     */
    fn fmt_meteo_rinex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(meteo) = &self.meteo {
            /*
             * List of observables
             */
            let mut descriptor = String::new();
            descriptor.push_str(&format!("{:6}", meteo.codes.len()));
            for (i, observable) in meteo.codes.iter().enumerate() {
                if (i % 9) == 0 && i > 0 {
                    descriptor.push_str("      "); // TAB
                }
                descriptor.push_str(&format!("    {}", observable));
            }
            writeln!(f, "{}", fmt_rinex(&descriptor, "# / TYPES OF OBSERV"))?;
            for sensor in &meteo.sensors {
                write!(f, "{}", sensor)?;
            }
        }
        Ok(())
    }
    /*
     * Observation Data fields formatting
     */
    fn fmt_observation_rinex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(obs) = &self.obs {
            if let Some(e) = obs.timeof_first_obs {
                let (y, m, d, hh, mm, ss, nanos) =
                    (e + e.leap_seconds(true).unwrap_or(0.0) * Unit::Second).to_gregorian_utc();
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(
                        &format!(
                            "  {:04}    {:02}    {:02}    {:02}    {:02}   {:02}.{:07}     {:x}",
                            y, m, d, hh, mm, ss, nanos, e.time_scale
                        ),
                        "TIME OF FIRST OBS"
                    )
                )?;
            }
            if let Some(e) = obs.timeof_last_obs {
                let (y, m, d, hh, mm, ss, nanos) =
                    (e + e.leap_seconds(true).unwrap_or(0.0) * Unit::Second).to_gregorian_utc();
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(
                        &format!(
                            "  {:04}    {:02}    {:02}    {:02}    {:02}   {:02}.{:07}     {:x}",
                            y, m, d, hh, mm, ss, nanos, e.time_scale
                        ),
                        "TIME OF LAST OBS"
                    )
                )?;
            }
            /*
             * Form the observables list
             */
            match self.version.major {
                1 | 2 => {
                    /*
                     * List of observables
                     */
                    let mut descriptor = String::new();
                    if let Some((_constell, observables)) = obs.codes.iter().next() {
                        descriptor.push_str(&format!("{:6}", observables.len()));
                        for (i, observable) in observables.iter().enumerate() {
                            if (i % 9) == 0 && i > 0 {
                                descriptor.push_str("      "); // TAB
                            }
                            descriptor.push_str(&format!("    {}", observable));
                        }
                        writeln!(f, "{}", fmt_rinex(&descriptor, "# / TYPES OF OBSERV"))?;
                    }
                },
                _ => {
                    /*
                     * List of observables
                     */
                    for (constell, observables) in &obs.codes {
                        let mut descriptor = String::new();
                        descriptor.push_str(&format!("{:x}{:5}", constell, observables.len()));
                        for (i, observable) in observables.iter().enumerate() {
                            if (i % 13) == 0 && (i > 0) {
                                descriptor.push_str("        "); // TAB
                            }
                            descriptor.push_str(&format!(" {}", observable)); // TAB
                        }
                        writeln!(f, "{}", fmt_rinex(&descriptor, "SYS / # / OBS TYPES"))?;
                    }
                },
            }
            // must take place after list of observables:
            //  TODO DCBS compensations
            //  TODO PCVs compensations
        }
        Ok(())
    }
    /*
     * Format all comments
     */
    pub(crate) fn fmt_comments(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for comment in self.comments.iter() {
            writeln!(f, "{}", fmt_comment(comment))?;
        }
        Ok(())
    }
    /*
     * Parse IONEX grid
     */
    fn parse_grid(line: &str) -> Result<Linspace, ParsingError> {
        let mut start = 0.0_f64;
        let mut end = 0.0_f64;
        let mut spacing = 0.0_f64;
        for (index, item) in line.split_ascii_whitespace().enumerate() {
            let item = item.trim();
            match index {
                0 => {
                    start = f64::from_str(item).or(Err(ParsingError::IonexGridSpecs))?;
                },
                1 => {
                    end = f64::from_str(item).or(Err(ParsingError::IonexGridSpecs))?;
                },
                2 => {
                    spacing = f64::from_str(item).or(Err(ParsingError::IonexGridSpecs))?;
                },
                _ => {},
            }
        }
        if spacing == 0.0 {
            // avoid linspace verification in this case
            Ok(Linspace {
                start,
                end,
                spacing,
            })
        } else {
            let grid = Linspace::new(start, end, spacing)?;
            Ok(grid)
        }
    }
    /*
     * Parse list of observables (V2)
     */
    fn parse_v2_observables(
        line: &str,
        constell: Option<Constellation>,
        meteo: &mut MeteoHeader,
        observation: &mut ObservationHeader,
    ) {
        lazy_static! {
            /*
             *  Only GPS, Glonass, Galileo and SBAS are supported in V2 RINEX
             */
            static ref KNOWN_V2_CONSTELLS: [Constellation; 4] = [
                Constellation::GPS,
                Constellation::SBAS,
                Constellation::Glonass,
                Constellation::Galileo,
            ];
        }
        let line = line.split_at(6).1;
        for item in line.split_ascii_whitespace() {
            if let Ok(obs) = Observable::from_str(item.trim()) {
                match constell {
                    Some(Constellation::Mixed) => {
                        for constell in KNOWN_V2_CONSTELLS.iter() {
                            if let Some(codes) = observation.codes.get_mut(constell) {
                                codes.push(obs.clone());
                            } else {
                                observation.codes.insert(*constell, vec![obs.clone()]);
                            }
                        }
                    },
                    Some(c) => {
                        if let Some(codes) = observation.codes.get_mut(&c) {
                            codes.push(obs.clone());
                        } else {
                            observation.codes.insert(c, vec![obs.clone()]);
                        }
                    },
                    None => meteo.codes.push(obs),
                }
            }
        }
    }
    /*
     * Parse list of observables (V3)
     */
    fn parse_v3_observables(
        line: &str,
        current_constell: &mut Option<Constellation>,
        observation: &mut ObservationHeader,
    ) {
        let (possible_counter, items) = line.split_at(6);
        if !possible_counter.is_empty() {
            let code = &possible_counter[..1];
            if let Ok(c) = Constellation::from_str(code) {
                *current_constell = Some(c);
            }
        }
        if let Some(constell) = current_constell {
            // system correctly identified
            for item in items.split_ascii_whitespace() {
                if let Ok(observable) = Observable::from_str(item) {
                    if let Some(codes) = observation.codes.get_mut(constell) {
                        codes.push(observable);
                    } else {
                        observation.codes.insert(*constell, vec![observable]);
                    }
                }
            }
        }
    }
    /*
     * Parse list of DORIS observables
     */
    fn parse_doris_observables(line: &str, doris: &mut DorisHeader) {
        let items = line.split_at(6).1;
        for item in items.split_ascii_whitespace() {
            if let Ok(observable) = Observable::from_str(item) {
                doris.observables.push(observable);
            }
        }
    }
}

impl std::fmt::Display for Header {
    /// `Header` formatter, mainly for RINEX file production purposes
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // start with CRINEX attributes, if need be
        if let Some(obs) = &self.obs {
            if let Some(crinex) = &obs.crinex {
                writeln!(f, "{}", crinex)?;
            }
        }

        self.fmt_rinex_version_type(f)?;
        self.fmt_comments(f)?;

        // PGM / RUN BY / DATE
        writeln!(
            f,
            "{}",
            fmt_rinex(
                &format!("{:<20}{:<20}{:<20}", self.program, self.run_by, self.date),
                "PGM / RUN BY / DATE"
            )
        )?;

        // OBSERVER / AGENCY
        writeln!(
            f,
            "{}",
            fmt_rinex(
                &format!("{:<20}{}", self.observer, self.agency),
                "OBSERVER /AGENCY"
            )
        )?;

        if let Some(marker) = &self.geodetic_marker {
            writeln!(f, "{}", fmt_rinex(&marker.name, "MARKER NAME"))?;
            if let Some(number) = marker.number() {
                writeln!(f, "{}", fmt_rinex(&number, "MARKER NUMBER"))?;
            }
        }

        // APRIORI POS
        if let Some(position) = self.ground_position {
            writeln!(
                f,
                "{}",
                fmt_rinex(&format!("{:X}", position), "APPROX POSITION XYZ")
            )?;
        }

        // ANT
        if let Some(antenna) = &self.rcvr_antenna {
            writeln!(
                f,
                "{}",
                fmt_rinex(
                    &format!("{:<20}{}", antenna.model, antenna.sn),
                    "ANT # / TYPE"
                )
            )?;
            if let Some(coords) = &antenna.coords {
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(
                        &format!("{:14.4}{:14.4}{:14.4}", coords.0, coords.1, coords.2),
                        "APPROX POSITION XYZ"
                    )
                )?;
            }
            writeln!(
                f,
                "{}",
                fmt_rinex(
                    &format!(
                        "{:14.4}{:14.4}{:14.4}",
                        antenna.height.unwrap_or(0.0),
                        antenna.eastern.unwrap_or(0.0),
                        antenna.northern.unwrap_or(0.0)
                    ),
                    "ANTENNA: DELTA H/E/N"
                )
            )?;
        }
        // RCVR
        if let Some(rcvr) = &self.rcvr {
            writeln!(
                f,
                "{}",
                fmt_rinex(
                    &format!("{:<20}{:<20}{}", rcvr.sn, rcvr.model, rcvr.firmware),
                    "REC # / TYPE / VERS"
                )
            )?;
        }
        // INTERVAL
        if let Some(interval) = &self.sampling_interval {
            writeln!(
                f,
                "{}",
                fmt_rinex(&format!("{:6}", interval.to_seconds()), "INTERVAL")
            )?;
        }

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

        // RINEX Type dependent header
        self.fmt_rinex_dependent(f)?;

        //TODO
        // things that could be nice to squeeze in:
        // [+] SBAS contained (detailed vehicles)
        // [+] RINEX 3 -> 2 observables conversion (see OBS/V2/rovn as an example)
        writeln!(f, "{}", fmt_rinex("", "END OF HEADER"))
    }
}

impl Header {
    /// Generate special FILE MERGE comment, to mark [Header] as merged.
    pub(crate) fn merge_comment(timestamp: Epoch) -> String {
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

#[cfg(feature = "qc")]
impl Render for Header {
    fn render(&self) -> Markup {
        html! {
            tr {
                th { "Antenna" }
                @if let Some(antenna) = &self.rcvr_antenna {
                    td { (antenna.render()) }
                } @else {
                    td { "No information" }
                }
            }
            tr {
                th { "Receiver" }
                @ if let Some(rcvr) = &self.rcvr {
                    td { (rcvr.render()) }
                } else {
                    td { "No information" }
                }
            }
        }
    }
}

#[cfg(feature = "processing")]
fn header_mask_eq(hd: &mut Header, _item: &FilterItem) {}

#[cfg(feature = "processing")]
pub(crate) fn header_mask_mut(hd: &mut Header, f: &MaskFilter) {
    match f.operand {
        MaskOperand::Equals => header_mask_eq(hd, &f.item),
        MaskOperand::NotEquals => {},
        MaskOperand::GreaterThan => {},
        MaskOperand::GreaterEquals => {},
        MaskOperand::LowerThan => {},
        MaskOperand::LowerEquals => {},
    }
    if let Some(obs) = &mut hd.obs {
        obs.mask_mut(f);
    }
    if let Some(met) = &mut hd.meteo {
        met.mask_mut(f);
    }
    if let Some(ionex) = &mut hd.ionex {
        ionex.mask_mut(f);
    }
    if let Some(doris) = &mut hd.doris {
        doris.mask_mut(f);
    }
}
