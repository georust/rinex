//! Monument / Geodetic marker frame
/**
 * Geodetic / Monument marker
 * Does not have subrecord (simple record)
 **/
use crate::{
    message::time::{decode_gpst_epoch, TimeResolution},
    Error,
};

use hifitime::Epoch;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Record {
    /// [Epoch]
    pub epoch: Epoch,
    /// Antenna position in ECEF [m]
    pub antenna_ecef_xyz: (f64, f64, f64),
    /// Antenna ENU offset [m]
    pub antenna_offset_enu: (f64, f64, f64),
}

impl Record {
    /// Builds new [Frame] at specific [Epoch] and position
    /// as latitude, longitude, altitude above sea level
    pub fn new(
        epoch: Epoch,
        antenna_lat_ddeg: f64,
        antenna_long_ddeg: f64,
        antenna_alt_sea_m: f64,
        antenna_offset_enu: (f64, f64, f64),
    ) -> Self {
        Self {
            epoch,
            antenna_ecef_xyz: (antenna_lat_ddeg, antenna_long_ddeg, antenna_alt_sea_m),
            antenna_offset_enu,
        }
    }
    /// [Self] Decoding attempt from buffered content witn [TimeResolution] specs
    pub(crate) fn decode(mlen: usize, time_res: TimeResolution, buf: &[u8]) -> Result<Self, Error> {
        Ok(Self::default())
    }
    /// Encodes [Self] into buffer, returns encoded size (total bytes).
    /// [Self] must fit in preallocated buffer.
    pub(crate) fn encode(&self, buf: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn bnx00_monument_error() {
        let buf = [0, 0, 0, 0];
        let mlen = 4;
        let time_res = TimeResolution::QuarterSecond;
        let monument = Record::decode(mlen, time_res, &buf);
        assert!(monument.is_err());
    }
    #[test]
    fn bnx00_monument_decoding() {
        let buf = [0, 0, 0, 0];
        let mlen = 4;
        let time_res = TimeResolution::QuarterSecond;
        let monument = Record::decode(mlen, time_res, &buf);
        assert!(monument.is_ok());
        let monument = monument.unwrap();
        assert_eq!(monument.epoch, Epoch::from_gpst_seconds(10.0));
    }
}
