//! Monument Geodetic marker specific frames
use crate::{
    message::{record::solutions::FieldID, Message},
    prelude::TimeScale,
    utils::Utils,
    Error,
};

use std::str::from_utf8;

mod ecef;
mod geo;
mod temporal;

pub use ecef::{PositionEcef3d, Velocity3d};
pub use geo::{PositionGeo3d, VelocityNED3d};
pub use temporal::TemporalSolution;

#[derive(Debug, Clone, PartialEq)]
pub enum SolutionsFrame {
    /// Comment
    Comment(String),
    /// New ECEF position vector
    AntennaEcefPosition(PositionEcef3d),
    /// New ECEF velocity vector
    AntennaEcefVelocity(Velocity3d),
    /// New geodetic vector
    AntennaGeoPosition(PositionGeo3d),
    /// New local velocity vector
    AntennaGeoVelocity(VelocityNED3d),
    /// Time System (announcement or redefinition)
    TimeSystem(TimeScale),
    /// Temporal Solution
    TemporalSolution(TemporalSolution),
    /// Extra note
    Extra(String),
}

impl SolutionsFrame {
    /// Builds new [SolutionsFrame::Comment]
    pub fn new_comment(comment: &str) -> Self {
        Self::Comment(comment.to_string())
    }
    /// Builds new [SolutionsFrame::AntennaEcefPosition] expressed in WGS84
    pub fn new_antenna_wgs84_ecef_position(x_ecef_m: f64, y_ecef_m: f64, z_ecef_m: f64) -> Self {
        Self::AntennaEcefPosition(PositionEcef3d::new_wgs84(x_ecef_m, y_ecef_m, z_ecef_m))
    }
    /// Builds new [SolutionsFrame::AntennaEcefPosition]
    pub fn new_antenna_ecef_position(
        x_ecef_m: f64,
        y_ecef_m: f64,
        z_ecef_m: f64,
        ellipsoid: &str,
    ) -> Self {
        Self::AntennaEcefPosition(PositionEcef3d {
            x_ecef_m,
            y_ecef_m,
            z_ecef_m,
            ellipsoid: ellipsoid.to_string(),
        })
    }
    /// Builds new [SolutionsFrame::AntennaEcefVelocity]
    pub fn new_antenna_velocity(x_m_s: f64, y_m_s: f64, z_m_s: f64) -> Self {
        Self::AntennaEcefVelocity(Velocity3d {
            x_m_s,
            y_m_s,
            z_m_s,
        })
    }
    /// Builds new [SolutionsFrame::AntennaGeoPosition] expressed in WGS84
    pub fn new_antenna_wgs84_geo_position(long_ddeg: f64, lat_ddeg: f64, alt_m: f64) -> Self {
        Self::AntennaGeoPosition(PositionGeo3d::new_wgs84(long_ddeg, lat_ddeg, alt_m))
    }
    /// Builds new [SolutionsFrame::AntennaGeoPosition]
    pub fn new_antenna_geo_position(
        long_ddeg: f64,
        lat_ddeg: f64,
        alt_m: f64,
        ellipsoid: &str,
    ) -> Self {
        Self::AntennaGeoPosition(PositionGeo3d {
            long_ddeg,
            lat_ddeg,
            alt_m,
            ellipsoid: ellipsoid.to_string(),
        })
    }
    /// Builds new [SolutionsFrame::AntennaGeoVelocity]
    pub fn new_antenna_geo_ned_velocity(north_m_s: f64, east_m_s: f64, up_m_s: f64) -> Self {
        Self::AntennaGeoVelocity(VelocityNED3d {
            north_m_s,
            east_m_s,
            up_m_s,
        })
    }
    /// Announce or redefine the [TimeScale] in which the Clock stete
    /// (temporal solution) is expressed in
    pub fn new_timescale(ts: TimeScale) -> Self {
        Self::TimeSystem(ts)
    }
    /// Update the temporal solution
    pub fn new_temporal_solution(sol: TemporalSolution) -> Self {
        Self::TemporalSolution(sol)
    }

    /// Returns total length (bytewise) required to fully encode [SolutionsFrame]..
    /// Use this to fulfill [Self::encode] requirements.
    pub(crate) fn encoding_size(&self) -> usize {
        match self {
            Self::TemporalSolution(sol) => {
                sol.encoding_size() + 1 // FID
            },
            Self::AntennaEcefPosition(pos) => {
                pos.encoding_size() + 1 // FID
            },
            Self::AntennaGeoPosition(pos) => {
                pos.encoding_size() + 1 // FID
            },
            Self::Comment(s) | Self::Extra(s) => {
                let s_len = s.len();
                s_len + Message::bnxi_encoding_size(s_len as u32) + 1 // FID
            },
            Self::TimeSystem(_) => 2,
            Self::AntennaEcefVelocity(_) | Self::AntennaGeoVelocity(_) => 25,
        }
    }

    /// Returns expected [FieldID] for [Self]
    pub(crate) fn to_field_id(&self) -> FieldID {
        match self {
            Self::Extra(_) => FieldID::Extra,
            Self::Comment(_) => FieldID::Comment,
            Self::AntennaGeoPosition(_) => FieldID::AntennaGeoPosition,
            Self::AntennaEcefPosition(_) => FieldID::AntennaEcefPosition,
            Self::AntennaGeoVelocity(_) => FieldID::AntennaGeoVelocity,
            Self::AntennaEcefVelocity(_) => FieldID::AntennaEcefVelocity,
            Self::TimeSystem(_) => FieldID::ReceiverTimeSystem,
            Self::TemporalSolution(sol) => {
                if sol.drift_s_s.is_some() {
                    FieldID::ReceiverClockOffsetDrift
                } else {
                    FieldID::ReceiverClockOffset
                }
            },
        }
    }

    /// [MonumentGeoFrame] decoding attempt from given [FieldID]
    pub(crate) fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        let buf_len = buf.len();
        if buf_len < 2 {
            // smallest size
            return Err(Error::NotEnoughBytes);
        }

        // decode FID
        let (fid, mut ptr) = Message::decode_bnxi(buf, big_endian);
        let fid = FieldID::from(fid);

        match fid {
            FieldID::Comment | FieldID::Extra => {
                if buf_len < ptr + 1 {
                    // can't decode 1-4b
                    return Err(Error::NotEnoughBytes);
                }

                // decode slen
                let (s_len, size) = Message::decode_bnxi(&buf[ptr..], big_endian);
                let s_len = s_len as usize;
                ptr += size;

                if buf_len < ptr + s_len {
                    return Err(Error::NotEnoughBytes); // can't parse entire string
                }

                match from_utf8(&buf[ptr..ptr + s_len]) {
                    Ok(s) => match fid {
                        FieldID::Comment => Ok(Self::Comment(s.to_string())),
                        FieldID::Extra => Ok(Self::Extra(s.to_string())),
                        FieldID::AntennaEcefPosition
                        | FieldID::AntennaEcefVelocity
                        | FieldID::AntennaGeoPosition
                        | FieldID::AntennaGeoVelocity
                        | FieldID::ReceiverTimeSystem
                        | FieldID::ReceiverClockOffset
                        | FieldID::ReceiverClockOffsetDrift
                        | FieldID::Unknown => Err(Error::UnknownMessage),
                    },
                    Err(_) => Err(Error::Utf8Error),
                }
            },
            FieldID::AntennaEcefPosition => {
                let ecef = PositionEcef3d::decode(big_endian, &buf[ptr..])?;
                Ok(Self::AntennaEcefPosition(ecef))
            },
            FieldID::AntennaGeoPosition => {
                let geo = PositionGeo3d::decode(big_endian, &buf[ptr..])?;
                Ok(Self::AntennaGeoPosition(geo))
            },
            FieldID::AntennaEcefVelocity | FieldID::AntennaGeoVelocity => {
                if buf_len < 25 {
                    return Err(Error::NotEnoughBytes); // cant decode 8x3
                }

                let x = Utils::decode_f64(big_endian, &buf[1..])?;
                let y = Utils::decode_f64(big_endian, &buf[9..])?;
                let z = Utils::decode_f64(big_endian, &buf[17..])?;

                if fid == FieldID::AntennaGeoVelocity {
                    let ned3d = VelocityNED3d {
                        north_m_s: x,
                        east_m_s: y,
                        up_m_s: z,
                    };

                    Ok(Self::AntennaGeoVelocity(ned3d))
                } else {
                    let vel3d = Velocity3d {
                        x_m_s: x,
                        y_m_s: y,
                        z_m_s: z,
                    };

                    Ok(Self::AntennaEcefVelocity(vel3d))
                }
            },
            FieldID::ReceiverTimeSystem => {
                if buf_len < 2 {
                    return Err(Error::NotEnoughBytes);
                }
                match buf[1] {
                    0 => Ok(Self::TimeSystem(TimeScale::GPST)),
                    1 => Ok(Self::TimeSystem(TimeScale::GST)),
                    3 => Ok(Self::TimeSystem(TimeScale::BDT)),
                    _ => Err(Error::NonSupportedTimescale),
                }
            },
            FieldID::ReceiverClockOffset => {
                let sol = TemporalSolution::decode_without_drift(big_endian, &buf[1..])?;
                Ok(Self::TemporalSolution(sol))
            },
            FieldID::ReceiverClockOffsetDrift => {
                let sol = TemporalSolution::decode_with_drift(big_endian, &buf[1..])?;
                Ok(Self::TemporalSolution(sol))
            },
            _ => Err(Error::NonSupportedSubRecord),
        }
    }

    /// Encodes [Self] into buffer, returns encoded size (total bytes).
    /// [Self] must fit in preallocated buffer.
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        // encode FID
        let fid = self.to_field_id() as u32;
        let mut ptr = Message::encode_bnxi(fid, big_endian, buf)?;

        match self {
            Self::Comment(s) | Self::Extra(s) => {
                // encode strlen
                let s_len = s.len();
                let size = Message::encode_bnxi(s_len as u32, big_endian, &mut buf[ptr..])?;
                ptr += size;

                buf[ptr..ptr + s_len].clone_from_slice(s.as_bytes()); // utf8 encoding
            },
            Self::AntennaEcefPosition(ecef3d) => {
                let _ = ecef3d.encode(big_endian, &mut buf[ptr..])?;
            },
            Self::AntennaGeoPosition(geo3d) => {
                let _ = geo3d.encode(big_endian, &mut buf[ptr..])?;
            },
            Self::AntennaEcefVelocity(vel3d) => {
                // encode (x, y, z)
                let bytes = if big_endian {
                    vel3d.x_m_s.to_be_bytes()
                } else {
                    vel3d.x_m_s.to_le_bytes()
                };

                buf[ptr..ptr + 8].copy_from_slice(&bytes);

                let bytes = if big_endian {
                    vel3d.y_m_s.to_be_bytes()
                } else {
                    vel3d.y_m_s.to_le_bytes()
                };

                buf[ptr + 8..ptr + 16].copy_from_slice(&bytes);

                let bytes = if big_endian {
                    vel3d.z_m_s.to_be_bytes()
                } else {
                    vel3d.z_m_s.to_le_bytes()
                };

                buf[ptr + 16..ptr + 24].copy_from_slice(&bytes);
            },
            Self::AntennaGeoVelocity(ned3d) => {
                // encode (north, east, up)
                let bytes = if big_endian {
                    ned3d.north_m_s.to_be_bytes()
                } else {
                    ned3d.north_m_s.to_le_bytes()
                };

                buf[ptr..ptr + 8].copy_from_slice(&bytes);

                let bytes = if big_endian {
                    ned3d.east_m_s.to_be_bytes()
                } else {
                    ned3d.east_m_s.to_le_bytes()
                };

                buf[ptr + 8..ptr + 16].copy_from_slice(&bytes);

                let bytes = if big_endian {
                    ned3d.up_m_s.to_be_bytes()
                } else {
                    ned3d.up_m_s.to_le_bytes()
                };

                buf[ptr + 16..ptr + 24].copy_from_slice(&bytes);
            },
            Self::TemporalSolution(sol) => {
                let _ = sol.encode(big_endian, &mut buf[ptr..])?;
            },
            Self::TimeSystem(ts) => match ts {
                TimeScale::GPST => {
                    buf[ptr] = 0;
                },
                TimeScale::GST => {
                    buf[ptr] = 1;
                },
                TimeScale::BDT => {
                    buf[ptr] = 3;
                },
                _ => {
                    buf[ptr] = 255;
                },
            },
        }

        Ok(size)
    }
}

#[cfg(test)]
mod test {
    use super::{
        PositionEcef3d, PositionGeo3d, SolutionsFrame, TemporalSolution, TimeScale, Velocity3d,
        VelocityNED3d,
    };

    #[test]
    fn encoding_size() {
        let fr = SolutionsFrame::new_comment("test");
        assert_eq!(fr.encoding_size(), 6);
    }

    #[test]
    fn solutions_comment() {
        let frame = SolutionsFrame::Comment("Hello".to_string());
        assert_eq!(frame.encoding_size(), 5 + 2);

        let big_endian = true;
        let mut buf = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let size = frame.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, frame.encoding_size());
        assert_eq!(buf, [0, 5, b'H', b'e', b'l', b'l', b'o', 0, 0, 0, 0, 0, 0]);

        let decoded = SolutionsFrame::decode(big_endian, &buf).unwrap();
        assert_eq!(decoded, frame);
    }

    #[test]
    fn pvt_solution_ecef_wgs84() {
        let pos3d = PositionEcef3d::new_wgs84(1.0, 2.0, 3.0);
        let frame = SolutionsFrame::AntennaEcefPosition(pos3d);
        assert_eq!(frame.encoding_size(), 1 + 8 + 8 + 8 + 1);

        let big_endian = true;

        let mut buf = [0; 16];
        assert!(frame.encode(big_endian, &mut buf).is_err());

        let mut buf = [0; 64];
        let size = frame.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, frame.encoding_size());

        assert_eq!(
            buf,
            [
                1, 0, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
        );

        let decoded = SolutionsFrame::decode(big_endian, &buf).unwrap();
        assert_eq!(decoded, frame);

        let vel3d = Velocity3d {
            x_m_s: 1.0,
            y_m_s: 2.0,
            z_m_s: 3.0,
        };

        let frame = SolutionsFrame::AntennaEcefVelocity(vel3d);
        assert_eq!(frame.encoding_size(), 1 + 8 + 8 + 8);

        let big_endian = true;

        let mut buf = [0; 16];
        assert!(frame.encode(big_endian, &mut buf).is_err());

        let mut buf = [0; 64];
        let size = frame.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, frame.encoding_size());

        assert_eq!(
            buf,
            [
                3, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
        );

        let decoded = SolutionsFrame::decode(big_endian, &buf).unwrap();
        assert_eq!(decoded, frame);

        let frame = SolutionsFrame::TimeSystem(TimeScale::BDT);
        assert_eq!(frame.encoding_size(), 1 + 1);
    }

    #[test]
    fn pvt_solution_ecef_pz90() {
        let pos3d = PositionEcef3d {
            x_ecef_m: 1.0,
            y_ecef_m: 2.0,
            z_ecef_m: 3.0,
            ellipsoid: "PZ90".to_string(),
        };

        let frame = SolutionsFrame::AntennaEcefPosition(pos3d);
        assert_eq!(
            frame.encoding_size(),
            1 // FID
            +3*8 //data
            +1 // Ellips
            +"PZ90".len()
        );

        let big_endian = true;

        let mut buf = [0; 16];
        assert!(frame.encode(big_endian, &mut buf).is_err());

        let mut buf = [0; 64];
        let size = frame.encode(big_endian, &mut buf).unwrap();
        assert_eq!(size, frame.encoding_size());

        assert_eq!(
            buf,
            [
                1, 4, b'P', b'Z', b'9', b'0', 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0,
                64, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        );

        let decoded = SolutionsFrame::decode(big_endian, &buf).unwrap();
        assert_eq!(decoded, frame);
    }

    #[test]
    fn pvt_solution_geo_wgs84() {
        let geo3d = PositionGeo3d::new_wgs84(1.0, 2.0, 3.0);
        let frame = SolutionsFrame::AntennaGeoPosition(geo3d);
        assert_eq!(frame.encoding_size(), 1 + 8 + 8 + 8 + 1);

        let big_endian = true;

        let mut buf = [0; 16];
        assert!(frame.encode(big_endian, &mut buf).is_err());

        let mut buf = [0; 64];
        let size = frame.encode(big_endian, &mut buf).unwrap();
        assert_eq!(size, frame.encoding_size());

        assert_eq!(
            buf,
            [
                2, 0, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
        );

        let decoded = SolutionsFrame::decode(big_endian, &buf).unwrap();
        assert_eq!(decoded, frame);

        let ned3d = VelocityNED3d {
            north_m_s: 1.0,
            east_m_s: 2.0,
            up_m_s: 3.0,
        };

        let frame = SolutionsFrame::AntennaGeoVelocity(ned3d);
        assert_eq!(frame.encoding_size(), 1 + 8 + 8 + 8);

        let big_endian = true;

        let mut buf = [0; 16];
        assert!(frame.encode(big_endian, &mut buf).is_err());

        let mut buf = [0; 64];
        let size = frame.encode(big_endian, &mut buf).unwrap();
        assert_eq!(size, frame.encoding_size());

        assert_eq!(
            buf,
            [
                4, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
        );

        let decoded = SolutionsFrame::decode(big_endian, &buf).unwrap();
        assert_eq!(decoded, frame);
    }

    #[test]
    fn pvt_solution_geo_pz90() {
        let big_endian = true;

        let geo3d = PositionGeo3d {
            long_ddeg: 1.0,
            lat_ddeg: 2.0,
            alt_m: 3.0,
            ellipsoid: "PZ90".to_string(),
        };

        let frame = SolutionsFrame::AntennaGeoPosition(geo3d);
        assert_eq!(frame.encoding_size(), 1 + 8 + 8 + 8 + 1 + 4);

        let mut buf = [0; 16];
        assert!(frame.encode(big_endian, &mut buf).is_err());

        let mut buf = [0; 64];
        let size = frame.encode(big_endian, &mut buf).unwrap();
        assert_eq!(size, frame.encoding_size());

        assert_eq!(
            buf,
            [
                2, 4, b'P', b'Z', b'9', b'0', 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0,
                64, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        );

        let decoded = SolutionsFrame::decode(big_endian, &buf).unwrap();

        assert_eq!(decoded, frame);
    }

    #[test]
    fn temporal_solutions() {
        let mut buf = [0; 8];
        let big_endian = true;

        let frame = SolutionsFrame::TimeSystem(TimeScale::GST);
        assert_eq!(frame.encoding_size(), 1 + 1);

        frame.encode(big_endian, &mut buf).unwrap();
        assert_eq!(buf, [5, 1, 0, 0, 0, 0, 0, 0]);

        let decoded = SolutionsFrame::decode(big_endian, &buf).unwrap();
        assert_eq!(frame, decoded);

        let mut buf = [0; 32];
        let sol = TemporalSolution {
            offset_s: 1.0,
            drift_s_s: None,
        };

        let frame = SolutionsFrame::TemporalSolution(sol);
        assert_eq!(frame.encoding_size(), 8 + 1);

        frame.encode(big_endian, &mut buf).unwrap();

        assert_eq!(
            buf,
            [
                6, 63, 240, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ]
        );

        let decoded = SolutionsFrame::decode(big_endian, &buf).unwrap();
        assert_eq!(frame, decoded);

        let mut buf = [0; 32];
        let sol = TemporalSolution {
            offset_s: 1.0,
            drift_s_s: Some(2.0),
        };

        let frame = SolutionsFrame::TemporalSolution(sol);
        assert_eq!(frame.encoding_size(), 16 + 1);

        frame.encode(big_endian, &mut buf).unwrap();

        assert_eq!(
            buf,
            [
                7, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0
            ]
        );

        let decoded = SolutionsFrame::decode(big_endian, &buf).unwrap();
        assert_eq!(frame, decoded);
    }
}
