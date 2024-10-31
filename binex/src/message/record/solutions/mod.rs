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
    ///     Epoch,
    /// };
    ///     
    /// let t = Epoch::from_gpst_seconds(60.0 + 0.75);
    ///
    /// let record = Solutions::new(t)
    ///     .with_position_ecef((1.0, 2.0, 3.0)) // x(t), y(t), z(t)
    ///     .with_velocity_ecef((1.0, 2.0, 3.0)) // vx(t), vy(t), vz(t)
    ///     .with_comments("Simple comment")
    ///     .with_extra_info("Extra note");
    ///
    /// let mut buf = [0; 16];
    /// match record.encode(true, &mut buf) {
    ///     Ok(_) => {
    ///         panic!("encoding should have failed!");
    ///     },
    ///     Err(Error::NotEnoughBytes) => {
    ///         // This frame does not fit in this pre allocated buffer.
    ///         // You should always tie your allocations to .encoding_size() !
    ///     },
    ///     Err(e) => {
    ///         panic!("{} error should not have happened!", e);
    ///     },
    /// }
    ///
    /// let mut buf = [0; 128];
    /// let _ = record.encode(true, &mut buf)
    ///     .unwrap();
    /// ```
    pub fn new(epoch: Epoch) -> Self {
        Self {
            epoch,
            frames: Vec::with_capacity(8),
        }
    }

    /// [Self] decoding attempt from buffered content.
    /// ## Inputs
    ///    - mlen: message length in bytes
    ///    - time_res: [TimeResolution]
    ///    - big_endian: endianness
    ///    - buf: buffered content
    /// ## Outputs
    ///    - Ok: [Self]
    ///    - Err: [Error]
    pub fn decode(mlen: usize, big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
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
            // decode field id
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

    /// Encodes [Self] into buffer, returns encoded size (total bytes).
    /// [Self] must fit in preallocated buffer.
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        // encode tstamp
        let t = self.epoch.to_time_scale(TimeScale::GPST);
        let mut ptr = encode_epoch(t, TimeResolution::MilliSecond, big_endian, buf)?;

        // encode subrecords
        for fr in self.frames.iter() {
            let offs = fr.encode(big_endian, &mut buf[ptr..])?;
            ptr += offs;
        }

        Ok(size)
    }

    /// Returns total length (bytewise) required to fully encode [Self].
    /// Use this to fulfill [Self::encode] requirements.
    pub(crate) fn encoding_size(&self) -> usize {
        let mut size = 6; // tstamp + source_meta
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
        s.frames.push(SolutionsFrame::Extra(info.to_string()));
        s
    }

    /// Define a new PVT solution in ECEF
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
    }
}
