//! Raw, Decoded, Modern Ephemeris and ionosphere models
use crate::{message::Message, Error};

mod fid;
use fid::FieldID;

mod gps;
pub use gps::{GPSEphemeris, GPSRaw};

/// [EphemerisFrame] may describe raw, decoded GNSS
/// Ephemeris or Ionosphere model parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum EphemerisFrame {
    /// Raw (encoded) GPS frame as is.
    /// It did not go through a decoding & interpretation process.
    GPSRaw(GPSRaw),
    /// Decoded GPS Ephemeris
    GPS(GPSEphemeris),
}

impl EphemerisFrame {
    /// Returns total length (bytewise) required to fully encode [Self].
    /// Use this to fulfill [Self::encode] requirements.
    pub fn encoding_size(&self) -> usize {
        match self {
            Self::GPSRaw(_) => GPSRaw::encoding_size(),
            Self::GPS(_) => GPSEphemeris::encoding_size(),
        }
    }

    /// Returns expected [FieldID] for [Self]
    pub(crate) fn to_field_id(&self) -> FieldID {
        match self {
            Self::GPS(_) => FieldID::GPS,
            Self::GPSRaw(_) => FieldID::GPSRaw,
        }
    }

    /// [EphemerisFrame] decoding attempt from given [FieldID]
    pub(crate) fn decode(mlen: usize, big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        // decode field id
        let (bnxi, size) = Message::decode_bnxi(&buf, big_endian);
        let fid = FieldID::from(bnxi);
        println!("bnx01-eph fid={:?}", fid);

        match fid {
            FieldID::GPSRaw => {
                let fr = GPSRaw::decode(big_endian, &buf[size..])?;
                Ok(Self::GPSRaw(fr))
            },
            FieldID::GPS => {
                let fr = GPSEphemeris::decode(big_endian, &buf[size..])?;
                Ok(Self::GPS(fr))
            },
            _ => Err(Error::UnknownRecordFieldId),
        }
    }

    /// Encodes [Self] into buffer, returns encoded size (total bytes).
    /// [Self] must fit in preallocated buffer.
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        match self {
            Self::GPSRaw(r) => r.encode(big_endian, buf),
            Self::GPS(r) => r.encode(big_endian, buf),
        }
    }

    /// Creates new [GPSRaw] frame
    pub fn new_gps_raw(&self, raw: GPSRaw) -> Self {
        Self::GPSRaw(raw)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn gps_raw() {
        let raw: GPSRaw = GPSRaw::new();
        assert_eq!(GPSRaw::encoding_size(), 1 + 4 + 72);

        let big_endian = true;
        let mut buf = [0; 77];
        let size = raw.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, GPSRaw::encoding_size());
        assert_eq!(buf, [0; 77],);
    }
}
