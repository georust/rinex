//! Message ID from and to binary

/// MessageID stands for Record ID byte
/// and follows the Sync Byte
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum MessageID {
    /// Geodetic Marker, Site and Refenrece point info:
    /// Geodetic metadata
    SiteMonumentMarker = 0,
    /// Decode Ephemeris frame
    Ephemeris = 1,
    /// Observation, time tag and receiver info
    Observation = 2,
    /// Local Meteorological and Geophysical information
    Meteo = 3,
    /// Receiver info: BINEX specific
    ReceiverInfo = 4,
    /// Processed Solutions like PVT
    ProcessedSolutions = 5,
    // Receiver info prototype: BINEX specific
    ReceiverInfoPrototype = 125,
    /// Meteo prototype: BINEX specific
    MeteoPrototype = 126,
    /// Observation, time tag and receiver info prototype
    ObservationPrototype = 127,
    // Unknown / unsupported message
    #[default]
    Unknown = 0xffffffff,
}

impl From<u32> for MessageID {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::SiteMonumentMarker,
            1 => Self::Ephemeris,
            2 => Self::Observation,
            3 => Self::Meteo,
            4 => Self::ReceiverInfo,
            5 => Self::ProcessedSolutions,
            125 => Self::ReceiverInfoPrototype,
            126 => Self::MeteoPrototype,
            127 => Self::ObservationPrototype,
            _ => Self::Unknown,
        }
    }
}

impl From<MessageID> for u32 {
    fn from(val: MessageID) -> u32 {
        match val {
            MessageID::SiteMonumentMarker => 0,
            MessageID::Ephemeris => 1,
            MessageID::Observation => 2,
            MessageID::Meteo => 3,
            MessageID::ReceiverInfo => 4,
            MessageID::ProcessedSolutions => 5,
            MessageID::ReceiverInfoPrototype => 125,
            MessageID::MeteoPrototype => 126,
            MessageID::ObservationPrototype => 127,
            _ => 0xff,
        }
    }
}
