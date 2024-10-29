//! Solutions FID

/// [FieldID] describes the content to follow in Solutions frames
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldID {
    /// Comment: simple comment (readable string)
    /// about the Geodetic marker. Several RINEX comments
    /// are described by several BINEX Geodetic comments (repeated frames).
    Comment = 0,
    /// New ECEF position vector
    AntennaEcefPosition = 1,
    /// New Geodetic position
    AntennaGeoPosition = 2,
    /// New ECEF velocity vector
    AntennaEcefVelocity = 3,
    /// New local geodetic velocity state
    AntennaGeoVelocity = 4,
    /// Receiver system time
    ReceiverTimeSystem = 5,
    /// Receiver clock state estimate
    ReceiverClockOffset = 6,
    /// Receiver clock state estimate
    ReceiverClockOffsetDrift = 7,
    /// Extra / Additional information, very similar to [Self::Comment]
    Extra = 127,
    /// Unknown / Invalid
    Unknown = 0xffffffff,
}

impl From<u32> for FieldID {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Comment,
            1 => Self::AntennaEcefPosition,
            2 => Self::AntennaGeoPosition,
            3 => Self::AntennaEcefVelocity,
            4 => Self::AntennaGeoVelocity,
            5 => Self::ReceiverTimeSystem,
            6 => Self::ReceiverClockOffset,
            7 => Self::ReceiverClockOffsetDrift,
            127 => Self::Extra,
            _ => Self::Unknown,
        }
    }
}

impl From<FieldID> for u32 {
    fn from(val: FieldID) -> u32 {
        match val {
            FieldID::Comment => 0,
            FieldID::AntennaEcefPosition => 1,
            FieldID::AntennaGeoPosition => 2,
            FieldID::AntennaEcefVelocity => 3,
            FieldID::AntennaGeoVelocity => 4,
            FieldID::ReceiverTimeSystem => 5,
            FieldID::ReceiverClockOffset => 6,
            FieldID::ReceiverClockOffsetDrift => 7,
            FieldID::Extra => 127,
            FieldID::Unknown => 0xffffffff,
        }
    }
}
