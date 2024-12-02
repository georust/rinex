//! Record: Message content

use crate::message::MessageID;

mod ephemeris; // ephemeris frames
mod monument; // geodetic marker // ephemeris frames
mod solutions; // solutions frames

pub use ephemeris::{
    EphemerisFrame, GALEphemeris, GLOEphemeris, GPSEphemeris, GPSRaw, SBASEphemeris,
};

pub use monument::{GeoFieldId, GeoStringFrame, MonumentGeoMetadata, MonumentGeoRecord};

pub use solutions::{
    PositionEcef3d, PositionGeo3d, Solutions, SolutionsFrame, TemporalSolution, Velocity3d,
    VelocityNED3d,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    /// Geodetic Marker, Site and Reference point information.
    /// Includes Geodetic metadata.
    MonumentGeo(MonumentGeoRecord),
    /// Ephemeris Frame
    EphemerisFrame(EphemerisFrame),
    /// Solutions frame
    Solutions(Solutions),
}

impl From<MonumentGeoRecord> for Record {
    fn from(geo: MonumentGeoRecord) -> Self {
        Self::MonumentGeo(geo)
    }
}

impl From<EphemerisFrame> for Record {
    fn from(fr: EphemerisFrame) -> Self {
        Self::EphemerisFrame(fr)
    }
}

impl Default for Record {
    fn default() -> Self {
        Self::MonumentGeo(Default::default())
    }
}

impl Record {
    /// Builds new [MonumentGeoRecord]
    pub fn new_monument_geo(r: MonumentGeoRecord) -> Self {
        Self::MonumentGeo(r)
    }
    /// Builds new [EphemerisFrame]
    pub fn new_ephemeris_frame(fr: EphemerisFrame) -> Self {
        Self::EphemerisFrame(fr)
    }
    /// Builds new [Solutions]
    pub fn new_solutions(sol: Solutions) -> Self {
        Self::Solutions(sol)
    }
    /// [MonumentGeoRecord] unwrapping attempt
    pub fn as_monument_geo(&self) -> Option<&MonumentGeoRecord> {
        match self {
            Self::MonumentGeo(r) => Some(r),
            _ => None,
        }
    }
    /// [EphemerisFrame] unwrapping attempt
    pub fn as_ephemeris(&self) -> Option<&EphemerisFrame> {
        match self {
            Self::EphemerisFrame(fr) => Some(fr),
            _ => None,
        }
    }
    /// [Solutions] unwrapping attempt
    pub fn as_solutions(&self) -> Option<&Solutions> {
        match self {
            Self::Solutions(sol) => Some(sol),
            _ => None,
        }
    }
    /// Returns [MessageID] to associate to [Self] in stream header.
    pub(crate) fn to_message_id(&self) -> MessageID {
        match self {
            Self::EphemerisFrame(_) => MessageID::Ephemeris,
            Self::MonumentGeo(_) => MessageID::SiteMonumentMarker,
            Self::Solutions(_) => MessageID::ProcessedSolutions,
        }
    }

    /// Returns internal encoding size
    pub(crate) fn encoding_size(&self) -> usize {
        match self {
            Self::Solutions(sol) => sol.encoding_size(),
            Self::EphemerisFrame(fr) => fr.encoding_size(),
            Self::MonumentGeo(geo) => geo.encoding_size(),
        }
    }
}
