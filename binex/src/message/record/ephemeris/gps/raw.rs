//! Raw GPS Ephemeris
use crate::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct GPSRaw {
    uint1: u8,
    sint4: i32,
    bytes: Vec<u8>,
}

impl Default for GPSRaw {
    fn default() -> Self {
        Self {
            uint1: 0,
            sint4: 0,
            bytes: [0; 72].to_vec(),
        }
    }
}

impl GPSRaw {
    /// Builds new Raw GPS Ephemeris message
    pub fn new() -> Self {
        Self::default()
    }
    pub const fn encoding_size() -> usize {
        1 + 4 + 72
    }
    pub fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < Self::encoding_size() {
            return Err(Error::NotEnoughBytes);
        }

        let uint1 = buf[0];
        let sint4 = if big_endian {
            i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]])
        } else {
            i32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]])
        };

        Ok(Self {
            uint1,
            sint4,
            bytes: buf[5..72 - 5].to_vec(),
        })
    }
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = Self::encoding_size();
        if buf.len() < size {
            Err(Error::NotEnoughBytes)
        } else {
            buf[0] = self.uint1;

            let bytes = if big_endian {
                self.sint4.to_be_bytes()
            } else {
                self.sint4.to_le_bytes()
            };

            buf[1..5].copy_from_slice(&bytes);
            buf[5..].copy_from_slice(&self.bytes);
            Ok(size)
        }
    }
}
