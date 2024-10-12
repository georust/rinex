//! Monument / Geodetic marker frame
/**
 * Geodetic / Monument marker
 * Does not have subrecord (simple record)
 **/
use crate::{
    message::{
        time::{decode_gpst_epoch, TimeResolution},
        Message,
    },
    Error,
};

use hifitime::Epoch;

mod fid;
mod src;

// private
use fid::FieldID;

// public
pub use src::MonumentGeoMetadata;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct MonumentGeoRecord {
    /// [Epoch]
    pub epoch: Epoch,
    /// Source of this information
    pub source_meta: MonumentGeoMetadata,
    // /// Antenna position in ECEF [m]
    // pub antenna_ecef_xyz: (f64, f64, f64),
    // /// Antenna ENU offset [m]
    // pub antenna_offset_enu: (f64, f64, f64),
}

impl MonumentGeoRecord {
    /// 4 byte date uint4       }
    /// 1 byte qsec             } epoch
    /// 1 byte MonumentGeoMetadata
    /// 1 byte FID
    ///     if FID corresponds to a character string
    ///     the next 1-4 BNXI byte represent the number of bytes in the caracter string
    const MIN_SIZE: usize = 4 + 1 + 1 + 1;
    /// Builds new [MonumentGeoRecord] at specific [Epoch] with desired content
    pub fn new(
        epoch: Epoch,
        source_meta: MonumentGeoMetadata,
        // antenna_lat_ddeg: f64,
        // antenna_long_ddeg: f64,
        // antenna_alt_sea_m: f64,
        // antenna_offset_enu: (f64, f64, f64),
    ) -> Self {
        Self {
            epoch,
            source_meta,
            //antenna_ecef_xyz: (antenna_lat_ddeg, antenna_long_ddeg, antenna_alt_sea_m),
            //antenna_offset_enu,
        }
    }
    /// [Self] decoding attempt from buffered content.
    pub fn decode(
        msglen: usize,
        time_res: TimeResolution,
        big_endian: bool,
        buf: &[u8],
    ) -> Result<Self, Error> {
        if buf.len() < Self::MIN_SIZE {
            return Err(Error::NotEnoughBytes);
        }

        // decode timestamp
        let epoch = decode_gpst_epoch(big_endian, time_res, &buf[..5])?;

        // decode source meta
        let source_meta = MonumentGeoMetadata::from(buf[5]);

        // decode field id
        let fid = Message::decode_bnxi_u32(&buf[6..], big_endian);

        Ok(Self { epoch, source_meta })
    }
    /// Encodes [Self] into buffer, returns encoded size (total bytes).
    /// [Self] must fit in preallocated buffer.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn monument_marker_bnx00_error() {
        let buf = [0, 0, 0, 0];
        let mlen = 4;
        let time_res = TimeResolution::QuarterSecond;
        let monument = MonumentGeoRecord::decode(mlen, time_res, true, &buf);
        assert!(monument.is_err());
    }
    #[test]
    fn big_endian_monument_decoding() {
        let buf = [0, 0, 1, 1, 1, 2];
        let mlen = 5;
        let time_res = TimeResolution::QuarterSecond;
        match MonumentGeoRecord::decode(mlen, time_res, true, &buf) {
            Ok(monument) => {
                assert_eq!(
                    monument.epoch,
                    Epoch::from_gpst_seconds(256.0 * 60.0 + 60.0 + 0.25)
                );
                assert_eq!(monument.source_meta, MonumentGeoMetadata::IGS);
            },
            Err(e) => panic!("decoding error: {}", e),
        }
    }
}
