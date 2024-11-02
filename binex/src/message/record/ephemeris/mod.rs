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
            FieldID::GLO => {
                let fr = GLOEphemeris::decode(big_endian, &buf[size..])?;
                Ok(Self::GLO(fr))
            },
            FieldID::SBAS => {
                let fr = SBASEphemeris::decode(big_endian, &buf[size..])?;
                Ok(Self::SBAS(fr))
            },
            FieldID::GAL => {
                let fr: GALEphemeris = GALEphemeris::decode(big_endian, &buf[size..])?;
                Ok(Self::GAL(fr))
            },
            _ => Err(Error::NonSupportedMesssage(0)),
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
    pub fn new_gps_raw(raw: GPSRaw) -> Self {
        Self::GPSRaw(raw)
    }

    /// Creates new [GPSEphemeris] frame
    pub fn new_gps(gps: GPSEphemeris) -> Self {
        Self::GPS(gps)
    }

    /// Creates new [GLOEphemeris] frame
    pub fn new_glonass(glo: GLOEphemeris) -> Self {
        Self::GLO(glo)
    }

    /// Creates new [SBASEphemeris] frame
    pub fn new_sbas(sbas: SBASEphemeris) -> Self {
        Self::SBAS(sbas)
    }

    /// Creates new [GALEphemeris] frame
    pub fn new_galileo(gal: GALEphemeris) -> Self {
        Self::GAL(gal)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gps_eph() {
        let eph = GPSEphemeris {
            sqrt_a: 1.0,
            sv_health: 2,
            sv_prn: 3,
            toe: 4,
            tow: 5,
            toc: 6,
            tgd: 7.0,
            iodc: 8,
            clock_drift: 9.0,
            clock_drift_rate: 10.0,
            clock_offset: 11.0,
            iode: 13,
            delta_n_rad_s: 14.0,
            m0_rad: 15.0,
            e: 16.0,
            cic: 17.0,
            crc: 18.0,
            cis: 19.0,
            crs: 20.0,
            cuc: 21.0,
            cus: 22.0,
            omega_0_rad: 23.0,
            omega_dot_rad_s: 24.0,
            omega_rad: 25.0,
            i0_rad: 27.0,
            i_dot_rad_s: 28.0,
            ura_m: 29.0,
            uint2: 30,
        };

        assert_eq!(GPSEphemeris::encoding_size(), 128);

        let big_endian = true;

        let mut encoded = [0; 77];
        assert!(eph.encode(big_endian, &mut encoded).is_err());

        let mut encoded = [0; 128];
        let size = eph.encode(big_endian, &mut encoded).unwrap();
        assert_eq!(size, 128);

        let decoded = GPSEphemeris::decode(big_endian, &encoded).unwrap();

        assert_eq!(decoded, eph);
    }

    #[test]
    fn gps_raw() {
        let raw: GPSRaw = GPSRaw::default();
        assert_eq!(GPSRaw::encoding_size(), 1 + 1 + 4 + 72);

        let big_endian = true;

        let mut buf = [0; 78];
        let size = raw.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, GPSRaw::encoding_size());
        assert_eq!(buf, [0; 78]);

        let decoded = GPSRaw::decode(big_endian, &buf).unwrap();
        assert_eq!(decoded, raw);
    }

    #[test]
    fn gal() {
        let gal = GALEphemeris {
            toe_s: 1,
            sv_health: 2,
            sv_prn: 3,
            toe_week: 4,
            tow: 5,
            bgd_e5a_e1_s: 6.0,
            bgd_e5b_e1_s: 7.0,
            iodnav: 8,
            clock_drift: 9.0,
            clock_drift_rate: 10.0,
            clock_offset: 11.0,
            delta_n_semi_circles_s: 12.0,
            m0_rad: 12.0,
            e: 13.0,
            sqrt_a: 14.0,
            cic: 15.0,
            crc: 16.0,
            cis: 17.0,
            crs: 18.0,
            cus: 19.0,
            cuc: 20.0,
            omega_0_rad: 21.0,
            omega_dot_semi_circles: 22.0,
            omega_rad: 23.0,
            i0_rad: 25.0,
            idot_semi_circles_s: 26.0,
            sisa: 33.0,
            source: 34,
        };

        assert_eq!(GALEphemeris::encoding_size(), 128);

        let big_endian = true;

        let mut buf = [0; 128];
        let size = gal.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, GALEphemeris::encoding_size());

        let decoded = GALEphemeris::decode(big_endian, &buf).unwrap();
        assert_eq!(decoded, gal);
    }

    #[test]
    fn sbas() {
        let sbas = SBASEphemeris::default();
        assert_eq!(SBASEphemeris::encoding_size(), 98);

        let big_endian = true;

        let mut buf = [0; 98];
        let size = sbas.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, SBASEphemeris::encoding_size());
        assert_eq!(buf, [0; 98]);

        let decoded = SBASEphemeris::decode(big_endian, &buf).unwrap();
        assert_eq!(decoded, sbas);
    }
}
