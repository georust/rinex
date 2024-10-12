//! Record: Message content

use crate::message::MessageID;

mod monument; // geodetic marker
pub use monument::MonumentGeoRecord;

#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    /// Geodetic Marker, Site and Reference point information.
    /// Includes Geodetic metadata.
    MonumentGeo(MonumentGeoRecord),
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
    /// [MonumentGeoRecord] unwrapping attempt
    pub fn as_monument_geo(&self) -> Option<&MonumentGeoRecord> {
        match self {
            Self::MonumentGeo(r) => Some(r),
            _ => None,
        }
    }
    /// Returns [MessageID] to associate to [Self] in stream header.
    pub(crate) fn to_message_id(&self) -> MessageID {
        match self {
            Self::MonumentGeo(_) => MessageID::SiteMonumentMarker,
        }
    }
}
