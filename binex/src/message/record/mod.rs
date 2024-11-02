//! Record: Message content

use crate::message::MessageID;

mod ephemeris; // ephemeris frames
mod monument; // geodetic marker // ephemeris frames

pub use ephemeris::{
    EphemerisFrame, GALEphemeris, GLOEphemeris, GPSEphemeris, GPSRaw, SBASEphemeris,
};

pub use monument::{MonumentGeoMetadata, MonumentGeoRecord};

#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    /// Geodetic Marker, Site and Reference point information.
    /// Includes Geodetic metadata.
    MonumentGeo(MonumentGeoRecord),
    /// Ephemeris Frame
    EphemerisFrame(EphemerisFrame),
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
    /// Returns [MessageID] to associate to [Self] in stream header.
    pub(crate) fn to_message_id(&self) -> MessageID {
        match self {
            Self::EphemerisFrame(_) => MessageID::Ephemeris,
            Self::MonumentGeo(_) => MessageID::SiteMonumentMarker,
        }
    }

    /// Returns internal encoding size
    pub(crate) fn encoding_size(&self) -> usize {
        match self {
            Self::EphemerisFrame(fr) => fr.encoding_size(),
            Self::MonumentGeo(geo) => geo.encoding_size(),
        }
    }
}
