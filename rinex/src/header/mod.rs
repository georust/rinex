//! RINEX file Header

use crate::{
    antex::HeaderFields as AntexHeader,
    clock::HeaderFields as ClockHeader,
    doris::HeaderFields as DorisHeader,
    fmt_rinex,
    ground_position::GroundPosition,
    hardware::{Antenna, Receiver, SvAntenna},
    hatanaka::CRINEX,
    ionex::HeaderFields as IonexHeader,
    leap::Leap,
    marker::GeodeticMarker,
    meteo::HeaderFields as MeteoHeader,
    navigation::IonMessage,
    observation::HeaderFields as ObservationHeader,
    prelude::{Constellation, Duration, Epoch, COSPAR, SV},
    types::Type,
    version::Version,
};

use std::collections::HashMap;

mod formatting;
mod parsing;

#[cfg(feature = "qc")]
mod qc;

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "processing")]
pub(crate) mod processing;

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
    pub ionex: Option<IonexHeader>,
    /// DORIS RINEX specific fields
    #[cfg_attr(feature = "serde", serde(default))]
    pub doris: Option<DorisHeader>,
}

impl Header {
    /// Returns true if this [Header] containts the special [CRINEX] marker
    pub fn is_crinex(&self) -> bool {
        if let Some(obs) = &self.obs {
            obs.crinex.is_some()
        } else {
            false
        }
    }

    /// Builds a basic [Header] to describe a Multi-GNSS Navigation RINEX
    pub fn basic_nav() -> Self {
        Self::default()
            .with_type(Type::NavigationData)
            .with_constellation(Constellation::Mixed)
    }

    /// Builds a basic [Header] to describe a Multi-GNSS Observation RINEX
    pub fn basic_obs() -> Self {
        Self::default()
            .with_type(Type::ObservationData)
            .with_constellation(Constellation::Mixed)
    }

    /// Builds a basic [Header] to describe a Multi-GNSS Compressed Observation RINEX
    pub fn basic_crinex() -> Self {
        Self::default()
            .with_type(Type::ObservationData)
            .with_constellation(Constellation::Mixed)
            .with_crinex(CRINEX::default())
    }

    /// Generates the special "FILE MERGE" comment
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

    /// Copies and returns [Header] with specific RINEX [Version]
    pub fn with_version(&self, version: Version) -> Self {
        let mut s = self.clone();
        s.version = version;
        s
    }

    /// Copies and returns [Header] with specific RINEX [Type]
    pub fn with_type(&self, t: Type) -> Self {
        let mut s = self.clone();
        s.rinex_type = t;
        s
    }

    /// Copies and returns [Header] with generate information
    pub fn with_general_information(&self, program: &str, run_by: &str, agency: &str) -> Self {
        let mut s = self.clone();
        s.program = program.to_string();
        s.run_by = run_by.to_string();
        s.agency = agency.to_string();
        s
    }

    /// Copies and returns [Header] with special [CRINEX] fields
    pub fn with_crinex(&self, c: CRINEX) -> Self {
        let mut s = self.clone();
        if let Some(ref mut obs) = s.obs {
            obs.crinex = Some(c)
        }
        s
    }

    /// Copies and returns [Header] with specific [Receiver] information
    pub fn with_receiver(&self, r: Receiver) -> Self {
        let mut s = self.clone();
        s.rcvr = Some(r);
        s
    }

    /// Copies and returns [Header] with specific [Antenna] information
    pub fn with_receiver_antenna(&self, a: Antenna) -> Self {
        let mut s = self.clone();
        s.rcvr_antenna = Some(a);
        s
    }

    /// Copies and returns [Header] modified to [Constellation]
    pub fn with_constellation(&self, c: Constellation) -> Self {
        let mut s = self.clone();
        s.constellation = Some(c);
        s
    }

    /// Copies and returns [Header] with a new comment
    pub fn with_comment(&self, c: &str) -> Self {
        let mut s = self.clone();
        s.comments.push(c.to_string());
        s
    }

    /// Copies and returns [Header] with following comments
    pub fn with_comments(&self, c: Vec<String>) -> Self {
        let mut s = self.clone();
        s.comments = c.clone();
        s
    }

    /// Copies and returns [Header] with a specific [ObservationHeader]
    pub fn with_observation_fields(&self, fields: ObservationHeader) -> Self {
        let mut s = self.clone();
        s.obs = Some(fields);
        s
    }
}

impl std::fmt::Display for Header {

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
    }
}
