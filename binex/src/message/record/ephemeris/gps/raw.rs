//! Raw GPS Ephemeris
use crate::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct GPSRaw {
    pub svid1: u8,
    pub uint1: u8,
    pub sint4: i32,
    bytes: [u8; 72],
}

impl Default for GPSRaw {
    fn default() -> Self {
        Self {
            svid1: 0,
            uint1: 0,
            sint4: 0,
            bytes: [0; 72],
        }
    }
}

impl GPSRaw {
    /// Builds new Raw GPS Ephemeris message
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn encoding_size() -> usize {
        78
    }

    pub fn decode(big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < Self::encoding_size() {
            return Err(Error::NotEnoughBytes);
        }

        let svid1 = buf[0];
        let uint1 = buf[1];

        let sint4 = if big_endian {
            i32::from_be_bytes([buf[2], buf[3], buf[4], buf[5]])
        } else {
            i32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]])
        };

        let mut bytes = [0; 72];
        bytes.clone_from_slice(&buf[6..78]);

        Ok(Self {
            svid1,
            uint1,
            sint4,
            bytes,
        })
    }

    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = Self::encoding_size();
        if buf.len() < size {
            Err(Error::NotEnoughBytes)
        } else {
            buf[0] = self.svid1;
            buf[1] = self.uint1;

            let bytes = if big_endian {
                self.sint4.to_be_bytes()
            } else {
                self.sint4.to_le_bytes()
            };

            buf[2..6].copy_from_slice(&bytes);
            buf[6..78].copy_from_slice(&self.bytes);

            Ok(size)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gps_raw() {
        let big_endian = true;

        let buf = [0; 64];
        let decode = GPSRaw::decode(big_endian, &buf);
        assert!(decode.is_err());

        let mut buf = [0; 78];
        buf[0] = 10;
        buf[1] = 1;
        buf[6] = 10;
        buf[7] = 11;
        buf[6 + 71] = 123;

        let decoded = GPSRaw::decode(big_endian, &buf).unwrap();

        assert_eq!(decoded.svid1, 10);
        assert_eq!(decoded.uint1, 1);
        assert_eq!(decoded.bytes.len(), 72);

        assert_eq!(decoded.bytes[0], 10);
        assert_eq!(decoded.bytes[1], 11);
        assert_eq!(decoded.bytes[71], 123);

        let mut encoded = [0; 78];
        let size = decoded.encode(big_endian, &mut encoded).unwrap();
        assert_eq!(size, 78);
        assert_eq!(buf, encoded)
    }
}
