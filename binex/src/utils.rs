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
    /// u16 decoding attempt, as specified by
    /// [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html#uint2]
    pub fn decode_u16(big_endian: bool, buf: &[u8]) -> Result<u16, Error> {
        if buf.len() < 2 {
            Err(Error::NotEnoughBytes)
        } else {
            if big_endian {
                Ok(u16::from_be_bytes([buf[0], buf[1]]))
            } else {
                Ok(u16::from_le_bytes([buf[0], buf[1]]))
            }
        }
    }
    /// u32 decoding attempt, as specified by
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
    /// i32 decoding attempt, as specified by
    /// [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html#sint4]
    pub fn decode_i32(big_endian: bool, buf: &[u8]) -> Result<i32, Error> {
        if buf.len() < 4 {
            Err(Error::NotEnoughBytes)
        } else {
            if big_endian {
                Ok(i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]))
            } else {
                Ok(i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))
            }
        }
    }
    /// f32 decoding attempt, as specified by
    /// [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html#real4]
    pub fn decode_f32(big_endian: bool, buf: &[u8]) -> Result<f32, Error> {
        if buf.len() < 4 {
            Err(Error::NotEnoughBytes)
        } else {
            if big_endian {
                Ok(f32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]))
            } else {
                Ok(f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))
            }
        }
    }
    /// f64 decoding attempt, as specified by
    /// [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html#real8]
    pub fn decode_f64(big_endian: bool, buf: &[u8]) -> Result<f64, Error> {
        if buf.len() < 8 {
            Err(Error::NotEnoughBytes)
        } else {
            if big_endian {
                Ok(f64::from_be_bytes([
                    buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
                ]))
            } else {
                Ok(f64::from_le_bytes([
                    buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
                ]))
            }
        }
    }
}
