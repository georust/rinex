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
    /// Observation time tag and receiver info
    ObservationTimeTagRxInfo = 2,
    /// Local Meteorological and Geophysical information
    Meteo = 3,
    /// Receiver info: BINEX specific
    ReceiverInfo = 4,
    /// Processed Solutions like PVT
    ProcessedSolutions = 5,
    // Receiver info prototyping: BINEX specific
    ReceiverInfoPrototyping = 125,
    /// Meteo prototyping: BINEX specific
    MeteoPrototyping = 126,
    /// Observation time tag prototyping: BINEX specific
    ObservationTimeTagRxPrototyping = 127,
    // Unknown / unsupported message
    #[default]
    Unknown = 0xffffffff,
}

impl From<u32> for MessageID {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::SiteMonumentMarker,
            1 => Self::Ephemeris,
            2 => Self::ObservationTimeTagRxInfo,
            3 => Self::Meteo,
            4 => Self::ReceiverInfo,
            5 => Self::ProcessedSolutions,
            125 => Self::ReceiverInfoPrototyping,
            126 => Self::MeteoPrototyping,
            127 => Self::ObservationTimeTagRxPrototyping,
            _ => Self::Unknown,
        }
    }
}

impl From<MessageID> for u32 {
    fn from(val: MessageID) -> u32 {
        match val {
            MessageID::SiteMonumentMarker => 0,
            MessageID::Ephemeris => 1,
            // MessageID::ObservationTimeTagRxInfo => 0x02,
            // MessageID::Meteo => 0x03,
            // MessageID::ReceiverInfo => 0x04,
            // MessageID::ProcessedSolutions => 0x05,
            // MessageID::ReceiverInfoPrototyping => 0x7d,
            // MessageID::MeteoPrototyping => 0x7e,
            // MessageID::ObservationTimeTagRxPrototyping => 0x7f,
            _ => 0xffffffff,
        }
    }
}
