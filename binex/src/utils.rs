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
    pub fn decode_u32(buf: &[u8]) -> Result<u32, Error> {
        Ok(0)
    }
    /// U8 decoding attempt, as specified by
    /// [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html#uint1]
    pub fn decode_u8(buf: &[u8]) -> Result<u8, Error> {
        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use super::Utils;
    #[test]
    fn test_decode_u32() {}
}
