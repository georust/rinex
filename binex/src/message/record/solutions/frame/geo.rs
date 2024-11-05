use crate::{
    prelude::{Error, Message},
    utils::Utils,
};

use std::str::from_utf8;

#[derive(Debug, Clone, PartialEq)]
pub struct PositionGeo3d {
    /// longitude angle [ddeg]
    pub long_ddeg: f64,
    /// latitude angle [ddeg]
    pub lat_ddeg: f64,
    /// altitude above sea level [m]
    pub alt_m: f64,
    /// Ellpsoid shape
    pub ellipsoid: String,
}

impl PositionGeo3d {
    pub(crate) fn encoding_size(&self) -> usize {
        let mut size = 24;
        if !self.ellipsoid.eq("WGS84") {
            let str_len = self.ellipsoid.len();
            size += Message::bnxi_encoding_size(str_len as u32);
            size += str_len;
        } else {
            size += 1;
        }
        size
    }

    pub(crate) fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let mut ptr = 0;
        let size = self.encoding_size();

        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        // encode ellipsod
        if self.ellipsoid.eq("WGS84") {
            buf[ptr] = 0;
            ptr += 1;
        } else {
            let str_len = self.ellipsoid.len();

            let size = Message::encode_bnxi(str_len as u32, big_endian, &mut buf[ptr..])?;
            ptr += size;

            buf[ptr..ptr + str_len].copy_from_slice(self.ellipsoid.as_bytes());
            ptr += str_len;
        }

        // encode (x, y, z)
        let bytes = if big_endian {
            self.long_ddeg.to_be_bytes()
        } else {
            self.long_ddeg.to_le_bytes()
        };

        buf[ptr..ptr + 8].copy_from_slice(&bytes);

        let bytes = if big_endian {
            self.lat_ddeg.to_be_bytes()
        } else {
            self.lat_ddeg.to_le_bytes()
        };

        buf[ptr + 8..ptr + 16].copy_from_slice(&bytes);

        let bytes = if big_endian {
            self.alt_m.to_be_bytes()
        } else {
            self.alt_m.to_le_bytes()
        };

        buf[ptr + 16..ptr + 24].copy_from_slice(&bytes);
        Ok(size)
    }

    pub(crate) fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        let buf_len = buf.len();
        if buf_len < 2 {
            return Err(Error::NotEnoughBytes);
        }

        let (str_len, mut ptr) = Message::decode_bnxi(&buf, big_endian);
        let str_len = str_len as usize;

        let ellipsoid = if str_len == 0 {
            "WGS84".to_string()
        } else {
            let ellipsoid_str =
                from_utf8(&buf[ptr..ptr + str_len]).map_err(|_| Error::Utf8Error)?;

            ellipsoid_str.to_string()
        };

        ptr += str_len;

        if buf_len - str_len < 24 {
            return Err(Error::NotEnoughBytes);
        }

        let long_ddeg = Utils::decode_f64(big_endian, &buf[ptr..ptr + 8])?;
        let lat_ddeg = Utils::decode_f64(big_endian, &buf[ptr + 8..ptr + 16])?;
        let alt_m = Utils::decode_f64(big_endian, &buf[ptr + 16..ptr + 24])?;

        Ok(Self {
            long_ddeg,
            lat_ddeg,
            alt_m,
            ellipsoid,
        })
    }

    /// Defines new [PositionGeo3d] expressed in WGS84 ellipsoid.
    pub fn new_wgs84(long_ddeg: f64, lat_ddeg: f64, alt_m: f64) -> Self {
        Self {
            long_ddeg,
            lat_ddeg,
            alt_m,
            ellipsoid: "WGS84".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VelocityNED3d {
    /// Northern component
    pub north_m_s: f64,
    /// Eastern component
    pub east_m_s: f64,
    /// Upwards component
    pub up_m_s: f64,
}
