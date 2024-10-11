//! Message ID from and to binary

/// MessageID stands for Record ID byte
/// and follows the Sync Byte
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum MessageID {
    /// Geodetic Marker, Site and Refenrece point info:
    /// Geodetic metadata
    SiteMonumentMarker = 0x00,
    // /// Decode Ephemeris frame
    // Ephemeris = 0x01,
    // /// Observation time tag and receiver info
    // ObservationTimeTagRxInfo = 0x02,
    // /// Local Meteorological and Geophysical information
    // Meteo = 0x03,
    // /// Receiver info: BINEX specific
    // ReceiverInfo = 0x04,
    // /// Processed Solutions like PVT
    // ProcessedSolutions = 0x05,
    // /// Receiver info prototyping: BINEX specific
    // ReceiverInfoPrototyping = 0x7d,
    // /// Meteo prototyping: BINEX specific
    // MeteoPrototyping = 0x7e,
    // /// Observation time tag prototyping: BINEX specific
    // ObservationTimeTagRxPrototyping = 0x7f,
    // /// Unknown is used when building MessageID from buffer content
    #[default]
    Unknown = 0xff,
}

impl From<u8> for MessageID {
    fn from(val: u8) -> Self {
        match val {
            0x00 => Self::SiteMonumentMarker,
            // 0x01 => Self::Ephemeris,
            // 0x02 => Self::ObservationTimeTagRxInfo,
            // 0x03 => Self::Meteo,
            // 0x04 => Self::ReceiverInfo,
            // 0x05 => Self::ProcessedSolutions,
            // 0x7d => Self::ReceiverInfoPrototyping,
            // 0x7e => Self::MeteoPrototyping,
            // 0x7f => Self::ObservationTimeTagRxPrototyping,
            _ => Self::Unknown,
        }
    }
}

impl From<MessageID> for u8 {
    fn from(val: MessageID) -> u8 {
        match val {
            MessageID::SiteMonumentMarker => 0x00,
            // MessageID::Ephemeris => 0x01,
            // MessageID::ObservationTimeTagRxInfo => 0x02,
            // MessageID::Meteo => 0x03,
            // MessageID::ReceiverInfo => 0x04,
            // MessageID::ProcessedSolutions => 0x05,
            // MessageID::ReceiverInfoPrototyping => 0x7d,
            // MessageID::MeteoPrototyping => 0x7e,
            // MessageID::ObservationTimeTagRxPrototyping => 0x7f,
            _ => 0xff,
        }
    }
}
