//! Monument / Geodetic marker frames

use crate::{
    message::time::{decode_gpst_epoch, encode_epoch, TimeResolution},
    Error,
};

pub use frame::{PositionEcef3d, PositionGeo3d, TemporalSolution, Velocity3d, VelocityNED3d};
use hifitime::{Epoch, TimeScale};

mod fid;
mod frame;

// private
use fid::FieldID;

// public
pub use frame::SolutionsFrame;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Solutions {
    /// [Epoch]
    pub epoch: Epoch,
    /// Frames also refered to as Subrecords
    pub frames: Vec<SolutionsFrame>,
}

impl Iterator for Solutions {
    type Item = SolutionsFrame;
    fn next(&mut self) -> Option<Self::Item> {
        self.frames.iter().next().cloned()
    }
}

impl Solutions {
    /// 4 byte date uint4       }
    /// 2 byte ms               } epoch
    /// 1 byte FID
    ///     if FID corresponds to a character string
    ///     the next 1-4 BNXI byte represent the number of bytes in the caracter string
    /// follows: FID dependent sequence. See [FieldID].
    const MIN_SIZE: usize = 4 + 2 + 1;

    /// Creates new empty [Solutions], which is not suitable for encoding yet.
    /// Use other method to customize it!
    /// ```
    /// use binex::prelude::{
    ///     Epoch, Solutions, TemporalSolution,
    ///     Message, Record, Meta,
    /// };
    ///     
    /// // forge pvt solutions message
    /// let meta = Meta {
    ///     reversed: false,
    ///     big_endian: true,
    ///     enhanced_crc: false,
    /// };
    ///
    /// let t = Epoch::from_gpst_seconds(60.0 + 0.75);
    ///
    /// // Position Velocity Time (PVT) solution update
    /// let solutions = Solutions::new(t)
    ///     .with_pvt_ecef_wgs84(
    ///         1.0, // x(t) [m/s]
    ///         2.0, // y(t) [m/s]
    ///         3.0, // z(t) [m/s]
    ///         4.0, // dx(t)/dt [m/s^2]
    ///         5.0, // dy(t)/dt [m/s^2]
    ///         6.0, // dz(t)/dt [m/s^2]
    ///         TemporalSolution {
    ///             offset_s: 7.0, // [s]
    ///             drift_s_s: Some(8.0), // [s/s]
    ///         });
    ///
    /// let solutions = Record::new_solutions(solutions);
    /// let msg = Message::new(meta, solutions);
    ///
    /// let mut encoded = [0; 256];
    /// let _ = msg.encode(&mut encoded, msg.encoding_size())
    ///     .unwrap();
    ///
    /// let decoded = Message::decode(&encoded)
    ///     .unwrap();
    ///
    /// assert_eq!(decoded, msg);
    /// ```
    pub fn new(epoch: Epoch) -> Self {
        Self {
            epoch,
            frames: Vec::with_capacity(8),
        }
    }

    /// [Solutions] decoding attempt from buffered content.
    /// ## Inputs
    ///    - mlen: message length in bytes
    ///    - big_endian: endianness
    ///    - buf: buffered content
    /// ## Outputs
    ///    - Result<[Solutions], [Error]>
    pub(crate) fn decode(mlen: usize, big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if mlen < Self::MIN_SIZE {
            // does not look good
            return Err(Error::NotEnoughBytes);
        }

        // decode timestamp
        let epoch = decode_gpst_epoch(big_endian, TimeResolution::MilliSecond, &buf)?;

        // parse inner frames (= subrecords)
        let mut ptr = 6;
        let mut frames = Vec::<SolutionsFrame>::with_capacity(8);

        while ptr < mlen {
            // decode frame
            match SolutionsFrame::decode(big_endian, &buf[ptr..]) {
                Ok(fr) => {
                    ptr += fr.encoding_size();
                    frames.push(fr);
                },
                Err(_) => {
                    if ptr == 6 {
                        // did not parse a single record: incorrect message
                        return Err(Error::NotEnoughBytes);
                    } else {
                        break; // parsed all records
                    }
                },
            }
        }

        Ok(Self { epoch, frames })
    }

    /// Encodes [Solutions] into buffer, returns encoded size (total bytes).
    /// [Solutions] must fit in preallocated buffer.
    pub(crate) fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        // encode tstamp
        let t = self.epoch.to_time_scale(TimeScale::GPST);
        let mut ptr = encode_epoch(t, TimeResolution::MilliSecond, big_endian, buf)?;

        // encode subrecords
        for fr in self.frames.iter() {
            let size = fr.encode(big_endian, &mut buf[ptr..])?;
            ptr += size;
        }

        Ok(size)
    }

    /// Returns total length (bytewise) required to fully encode [Self].
    /// Use this to fulfill [Self::encode] requirements.
    pub(crate) fn encoding_size(&self) -> usize {
        let mut size = 6; // tstamp
        for fr in self.frames.iter() {
            size += fr.encoding_size(); // content
        }
        size
    }

    /// Add one [MonumentGeoFrame::Comment] to [MonumentGeoRecord].
    /// You can add as many as needed.
    pub fn with_comment(&self, comment: &str) -> Self {
        let mut s = self.clone();
        s.frames.push(SolutionsFrame::Comment(comment.to_string()));
        s
    }

    /// Attach readable Extra information to the solutions (like context description).
    pub fn with_extra_info(&self, info: &str) -> Self {
        let mut s = self.clone();
        // preserve unique item
        if let Some(info) = s
            .frames
            .iter_mut()
            .filter_map(|fr| match fr {
                SolutionsFrame::Extra(info) => Some(info),
                _ => None,
            })
            .reduce(|k, _| k)
        {
            *info = info.to_string(); // overwrite ; replace
        } else {
            s.frames.push(SolutionsFrame::Extra(info.to_string()));
        }
        s
    }

    /// Define PVT solution update, in ECEF
    pub fn with_pvt_ecef_wgs84(
        &self,
        x_ecef_m: f64,
        y_ecef_m: f64,
        z_ecef_m: f64,
        velx_ecef_m_s: f64,
        vely_ecef_m_s: f64,
        velz_ecef_m_s: f64,
        temporal_sol: TemporalSolution,
    ) -> Self {
        let mut s = self.clone();
        s.frames.push(SolutionsFrame::AntennaEcefPosition(
            PositionEcef3d::new_wgs84(x_ecef_m, y_ecef_m, z_ecef_m),
        ));
        s.frames
            .push(SolutionsFrame::AntennaEcefVelocity(Velocity3d {
                x_m_s: velx_ecef_m_s,
                y_m_s: vely_ecef_m_s,
                z_m_s: velz_ecef_m_s,
            }));
        s.frames
            .push(SolutionsFrame::TemporalSolution(temporal_sol));
        s
    }

    /// Define PVT solution update, in ellipsoid of your choice
    pub fn with_pvt_ecef(
        &self,
        x_ecef_m: f64,
        y_ecef_m: f64,
        z_ecef_m: f64,
        velx_ecef_m_s: f64,
        vely_ecef_m_s: f64,
        velz_ecef_m_s: f64,
        temporal_sol: TemporalSolution,
        ellipsoid: &str,
    ) -> Self {
        let mut s = self.clone();
        s.frames
            .push(SolutionsFrame::AntennaEcefPosition(PositionEcef3d {
                x_ecef_m,
                y_ecef_m,
                z_ecef_m,
                ellipsoid: ellipsoid.to_string(),
            }));
        s.frames
            .push(SolutionsFrame::AntennaEcefVelocity(Velocity3d {
                x_m_s: velx_ecef_m_s,
                y_m_s: vely_ecef_m_s,
                z_m_s: velz_ecef_m_s,
            }));
        s.frames
            .push(SolutionsFrame::TemporalSolution(temporal_sol));
        s
    }
}

#[cfg(test)]
mod test {
    use crate::{
        message::record::{Solutions, TemporalSolution},
        prelude::Epoch,
    };

    #[test]
    fn pvt_solutions_ecef() {
        let big_endian = true;

        let t0 = Epoch::from_gpst_seconds(71.000);

        let solutions = Solutions::new(t0).with_comment("Hello");

        let mut buf = [0; 32];
        let size = solutions.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, solutions.encoding_size());
        assert_eq!(
            buf,
            [
                0, 0, 0, 1, 0x2a, 0xf8, 0, 5, 72, 101, 108, 108, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
        );

        let decoded = Solutions::decode(size, big_endian, &buf).unwrap();

        assert_eq!(decoded, solutions);

        let solutions = solutions.with_pvt_ecef_wgs84(
            1.0,
            2.0,
            3.0,
            4.0,
            5.0,
            6.0,
            TemporalSolution {
                offset_s: 7.0,
                drift_s_s: None,
            },
        );

        let mut buf = [0; 128];
        let size = solutions.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, solutions.encoding_size());
        assert_eq!(
            buf,
            [
                0, 0, 0, 1, 0x2a, 0xf8, 0, 5, 72, 101, 108, 108, 111, 1, 0, 63, 240, 0, 0, 0, 0, 0,
                0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 3, 64, 16, 0, 0, 0, 0, 0, 0,
                64, 20, 0, 0, 0, 0, 0, 0, 64, 24, 0, 0, 0, 0, 0, 0, 6, 64, 28, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
        );

        let decoded = Solutions::decode(size, big_endian, &buf).unwrap();

        assert_eq!(decoded, solutions);
    }
}
