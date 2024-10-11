use crate::Error;

pub struct Utils;

impl Utils {
    /// Simple usize min() comparison to avoid `std` dependency.
    pub fn min_usize(a: usize, b: usize) -> usize {
        if a <= b {
            a
        } else {
            b
        }
    }
    /// U32 decoding attempt, as specified by
    /// [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html#uint4]
    pub fn decode_u32(big_endian: bool, buf: &[u8]) -> Result<u32, Error> {
        if buf.len() < 4 {
            Err(Error::NotEnoughBytes)
        } else {
            if big_endian {
                Ok(u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]))
            } else {
                Ok(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))
            }
        }
    }
}
