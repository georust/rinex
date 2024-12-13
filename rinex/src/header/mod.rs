//! RINEX file Header

use crate::{
    antex::HeaderFields as AntexHeader,
    clock::HeaderFields as ClockHeader,
    doris::HeaderFields as DorisHeader,
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
    /// RINEX [Version]
    pub version: Version,
    /// RINEX [Type]
    pub rinex_type: Type,
    /// GNSS [Constellation] describing this entire file.
    pub constellation: Option<Constellation>,
    /// Comments from this section
    pub comments: Vec<String>,
    /// Possible software name (publishing software)
    pub program: Option<String>,
    /// Possible software operator
    pub run_by: Option<String>,
    /// Possible date of publication
    pub date: Option<String>,
    /// Possible station / agency URL
    pub station_url: Option<String>,
    /// Name of observer / operator
    pub observer: Option<String>,
    /// Production Agency
    pub agency: Option<String>,
    /// Possible [GeodeticMarker]
    pub geodetic_marker: Option<GeodeticMarker>,
    /// Glonass FDMA channels
    pub glo_channels: HashMap<SV, i8>,
    /// Possible COSPAR number (launch information)
    pub cospar: Option<COSPAR>,
    /// Possible [Leap] seconds counter
    pub leap: Option<Leap>,
    /// Station approximate coordinates
    pub ground_position: Option<GroundPosition>,
    /// Optionnal wavelength correction factors
    pub wavelengths: Option<(u32, u32)>,
    /// Possible sampling interval
    pub sampling_interval: Option<Duration>,
    /// Possible file license
    pub license: Option<String>,
    /// Possible Digital Object Identifier
    pub doi: Option<String>,
    /// Optionnal GPS - UTC time difference
    pub gps_utc_delta: Option<u32>,
    /// Possible [Receiver] information
    #[cfg_attr(feature = "serde", serde(default))]
    pub rcvr: Option<Receiver>,
    /// Possible information about Receiver [Antenna]
    #[cfg_attr(feature = "serde", serde(default))]
    pub rcvr_antenna: Option<Antenna>,
    /// Possible information about satellite vehicle antenna.
    /// This only exists in ANTEX format.
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
            .with_observation_fields(ObservationHeader::default().with_crinex(CRINEX::default()))
    }

    /// Builds a basic [Header] for IONEX
    pub fn basic_ionex() -> Self {
        Self::default().with_ionex_fields(IonexHeader::default())
    }

    /// Generates the special "FILE MERGE" comment
    pub(crate) fn merge_comment(timestamp: Epoch) -> String {
        let (y, m, d, hh, mm, ss, _) = timestamp.to_gregorian_utc();
        format!(
            "geo-rust-{:<11} FILE MERGE          {}{}{} {}{}{} {:x}",
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
        s.program = Some(program.to_string());
        s.run_by = Some(run_by.to_string());
        s.agency = Some(agency.to_string());
        s
    }

    /// Copies and returns [Header] with special [CRINEX] fields
    pub(crate) fn with_crinex(&self, c: CRINEX) -> Self {
        let mut s = self.clone();
        if let Some(ref mut obs) = s.obs {
            obs.crinex = Some(c)
        } else {
            s.obs = Some(ObservationHeader::default().with_crinex(c));
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

    /// Copies and returns [Header] with specific [ObservationHeader]
    pub fn with_observation_fields(&self, fields: ObservationHeader) -> Self {
        let mut s = self.clone();
        s.obs = Some(fields);
        s
    }

    /// Copies and returns [Header] with specific [IonexHeader]
    pub fn with_ionex_fields(&self, fields: IonexHeader) -> Self {
        let mut s = self.clone();
        s.ionex = Some(fields);
        s
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{Epoch, Header};
    use std::str::FromStr;
    #[test]
    fn test_merge_comment() {
        let version = env!("CARGO_PKG_VERSION");
        let j2000 = Epoch::from_str("2000-01-01T00:00:00 UTC").unwrap();
        let comment = Header::merge_comment(j2000);
        // assert_eq!(
        //     comment,
        //     format!(
        //         "geo-rust-{:<11}   FILE MERGE  {}{}{} {}{}{} {:x}",
        //         y,
        //         m,
        //         d,
        //         hh,
        //         mm,
        //         ss,
        //         j2000.time_scale,
        //     ),
        // );
    }
}
