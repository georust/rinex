//! Raw, Decoded, Modern Ephemeris and ionosphere models
use crate::{message::Message, Error};

mod fid;
use fid::FieldID;

mod gps;
pub use gps::{GPSEphemeris, GPSRaw};

mod glonass;
pub use glonass::GLOEphemeris;

mod sbas;
pub use sbas::SBASEphemeris;

mod galileo;
pub use galileo::GALEphemeris;

/// [EphemerisFrame] may describe raw, decoded GNSS
/// Ephemeris or Ionosphere model parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum EphemerisFrame {
    /// Raw (encoded) GPS frame as is.
    /// It did not go through a decoding & interpretation process.
    GPSRaw(GPSRaw),
    /// Decoded GPS Ephemeris
    GPS(GPSEphemeris),
    /// Decoded Glonass Ephemeris
    GLO(GLOEphemeris),
    /// Decoded SBAS Ephemeris
    SBAS(SBASEphemeris),
    /// Decoded Galileo Ephemeris
    GAL(GALEphemeris),
}

impl EphemerisFrame {
    /// Returns total length (bytewise) required to fully encode [Self].
    /// Use this to fulfill [Self::encode] requirements.
    pub fn encoding_size(&self) -> usize {
        match self {
            Self::GPSRaw(_) => GPSRaw::encoding_size(),
            Self::GPS(_) => GPSEphemeris::encoding_size(),
            Self::GLO(_) => GLOEphemeris::encoding_size(),
            Self::SBAS(_) => SBASEphemeris::encoding_size(),
            Self::GAL(_) => GALEphemeris::encoding_size(),
        }
    }

    /// Returns expected [FieldID] for [Self]
    pub(crate) fn to_field_id(&self) -> FieldID {
        match self {
            Self::GPS(_) => FieldID::GPS,
            Self::GLO(_) => FieldID::GLO,
            Self::SBAS(_) => FieldID::SBAS,
            Self::GAL(_) => FieldID::GAL,
            Self::GPSRaw(_) => FieldID::GPSRaw,
        }
    }

    /// [EphemerisFrame] decoding attempt from given [FieldID]
    pub(crate) fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        // cant decode 1-4b
        if buf.len() < 1 {
            return Err(Error::NotEnoughBytes);
        }

        // decode FID
        let (bnxi, size) = Message::decode_bnxi(&buf, big_endian);
        let fid = FieldID::from(bnxi);

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
        if buf.len() < self.encoding_size() {
            return Err(Error::NotEnoughBytes);
        }

        // encode FID
        let fid = self.to_field_id() as u32;
        let offset = Message::encode_bnxi(fid, big_endian, buf)?;

        match self {
            Self::GPSRaw(r) => r.encode(big_endian, &mut buf[offset..]),
            Self::GPS(r) => r.encode(big_endian, &mut buf[offset..]),
            Self::GLO(r) => r.encode(big_endian, &mut buf[offset..]),
            Self::GAL(r) => r.encode(big_endian, &mut buf[offset..]),
            Self::SBAS(r) => r.encode(big_endian, &mut buf[offset..]),
        }
    }

    /// Creates new [GPSRaw] frame
    pub fn new_gps_raw(&self, raw: GPSRaw) -> Self {
        Self::GPSRaw(raw)
    }

    /// Creates new [GPSEphemeris] frame
    pub fn new_gps(&self, gps: GPSEphemeris) -> Self {
        Self::GPS(gps)
    }

    /// Creates new [GLOEphemeris] frame
    pub fn new_glonass(&self, glo: GLOEphemeris) -> Self {
        Self::GLO(glo)
    }

    /// Creates new [SBASEphemeris] frame
    pub fn new_sbas(&self, sbas: SBASEphemeris) -> Self {
        Self::SBAS(sbas)
    }

    /// Creates new [GALEphemeris] frame
    pub fn new_galileo(&self, gal: GALEphemeris) -> Self {
        Self::GAL(gal)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn gps_raw() {
        let mut eph = GPSEphemeris::default();
        eph.sv_prn = 10;
        eph.cic = 10.0;
        eph.cus = 12.0;
        eph.m0_rad = 100.0;
        eph.clock_offset = 123.0;

        assert_eq!(GPSEphemeris::encoding_size(), 153);

        let big_endian = true;

        let mut encoded = [0; 77];
        assert!(eph.encode(big_endian, &mut encoded).is_err());

        let mut encoded = [0; 153];
        let size = eph.encode(big_endian, &mut encoded).unwrap();
        assert_eq!(size, 153);

        let decoded = GPSEphemeris::decode(big_endian, &encoded).unwrap();

        assert_eq!(decoded, eph);
    }
    #[test]
    fn gps_eph() {
        let raw: GPSRaw = GPSRaw::new();
        assert_eq!(GPSRaw::encoding_size(), 1 + 4 + 72);

        let big_endian = true;
        let mut buf = [0; 77];
        let size = raw.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, GPSRaw::encoding_size());
        assert_eq!(buf, [0; 77]);
    }
}
