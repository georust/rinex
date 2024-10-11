//! Record: Message content

use crate::message::MessageID;

mod monument; // geodetic marker

pub use monument::Record as MonumentMarkerRecord;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Record {
    /// Geodetic Marker, Site and Refenrece point info:
    /// Geodetic metadata
    MonumentMarker(MonumentMarkerRecord),
}

impl Default for Record {
    fn default() -> Self {
        Self::MonumentMarker(Default::default())
    }
}

impl Record {
    /// Builds new [MonumentMarkerFrame]
    pub fn new_monument_marker(r: MonumentMarkerRecord) -> Self {
        Self::MonumentMarker(r)
    }
    /// [MonumentMarkerFrame] unwrapping attempt
    pub fn as_monument_marker(&self) -> Option<MonumentMarkerRecord> {
        match self {
            Self::MonumentMarker(r) => Some(*r),
            _ => None,
        }
    }
    /// Returns [MessageID] to associate to [Self]
    pub(crate) fn message_id(&self) -> MessageID {
        match self {
            Self::MonumentMarker(_) => MessageID::SiteMonumentMarker,
        }
    }
}
