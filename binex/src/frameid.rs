use crate::{Error, Message};

pub enum FrameID {
    GpsEphemeris0101 = 0x0101,
    Unknown = 0xffff,
}

impl From<u16> for FrameID {
    fn from(val: u16) -> FrameID {
        match val {
            0x101 => FrameID::GpsEphemeris0101,
            _ => FrameID::Unknown,
        }
    }
}

impl From<FrameID> for u16 {
    fn from(val: FrameID) -> u16 {
        match val {
            FrameID::GpsEphemeris0101 => 0x0101,
            FrameID::Unknown => 0xffff,
        }
    }
}

impl FrameID {
    /// Macro to improve reading process.
    /// Keep this to date with the database.
    pub(crate) const fn smallest_size(&self) -> usize {
        127
    }
    /// Macro to only attempt parsing messages contained in the database.
    /// Keep this to date with the database.
    pub(crate) const fn known_ids() -> [u16; 1] {
        [101]
    }
    const fn expected_size(&self) -> usize {
        match self {
            Self::GpsEphemeris0101 => 127,
            Self::Unknown => 0,
        }
    }
}

impl FrameID {
    pub(crate) fn decode_specific(&self, buf: &[u8], size: usize) -> Result<Message, Error> {
        if size < self.expected_size() {
            return Err(Error::NotEnoughBytes);
        }
        match self {
            Self::Unknown => Err(Error::UnknownFrame),
            Self::GpsEphemeris0101 => Self::decode_gpseph_1010(buf),
        }
    }
    fn decode_gpseph_1010(buf: &[u8]) -> Result<Message, Error> {
        Ok(Message {
            fid: FrameID::GpsEphemeris0101,
        })
    }
}
