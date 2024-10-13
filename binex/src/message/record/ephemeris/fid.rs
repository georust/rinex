//! Ephemeris Field ID

/// [FieldID] describes the content to follow in Ephemeris frames
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldID {
    /// Raw (non decoded) GPS Ephemeris message.
    /// Streamed as is: it did not go through the decoding process.
    ///   * uint1
    ///   * sint4 ToW in seconds
    ///   * 72 bytes: GPS ephemeris subframe
    GPSRaw = 0,
    /// Decoded GPS Ephemeris
    GPS = 1,
    /// Decoded GLO Ephemeris
    GLO = 2,
    /// Decoded SBAS Ephemeris
    SBAS = 3,
    /// Decoded GAL Ephemeris
    GAL = 4,
    /// Unknown / Invalid
    Unknown = 0xffffffff,
}

impl From<u32> for FieldID {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::GPSRaw,
            1 => Self::GPS,
            2 => Self::GLO,
            3 => Self::SBAS,
            4 => Self::GAL,
            _ => Self::Unknown,
        }
    }
}

impl From<FieldID> for u32 {
    fn from(val: FieldID) -> u32 {
        match val {
            FieldID::GPSRaw => 0,
            FieldID::GPS => 1,
            FieldID::GLO => 2,
            FieldID::SBAS => 3,
            FieldID::GAL => 4,
            FieldID::Unknown => 0xffffffff,
        }
    }
}
