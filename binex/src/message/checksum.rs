//! Checksum calculator
use md5::{Digest, Md5};

/// Checksum caculator
#[derive(Debug, Copy, Clone)]
pub enum Checksum {
    XOR8,
    XOR16,
    XOR32,
    MD5,
}

impl Checksum {
    // const fn binary_mask(&self) -> u32 {
    //     match self {
    //         Self::XOR8 => 0xff,
    //         Self::POLY12 => 0x7ff,
    //         Self::POLY20 => 0xfff,
    //     }
    // }
    // const fn look_up_table(&self) -> &[u8] {
    //     match self {
    //         Self::XOR8 => &[0, 0, 0, 0],
    //         Self::POLY12 => {
    //             // CRC12 table
    //             // x^16 + x^12 + x^5 +1
    //             &[0, 1, 2, 3]
    //         },
    //         Self::POLY20 => {
    //             // CRC16 table
    //             // x^32 + x^26 + x^23 + x^22 + x^16 + x^12 + x^11 + x^10 + x^8 + x^7 + x^5 + x^4 + x^2 + x^1 +1
    //             &[0, 1, 2, 3]
    //         },
    //     }
    // }
    /// Determines [ChecksumType] to use for a message
    pub fn from_len(mlen: usize, enhanced: bool) -> Self {
        if enhanced {
            if mlen < 128 {
                Self::XOR16
            } else if mlen < 1048575 {
                Self::XOR32
            } else {
                Self::MD5
            }
        } else {
            if mlen < 128 {
                Self::XOR8
            } else if mlen < 4096 {
                Self::XOR16
            } else if mlen < 1048575 {
                Self::XOR32
            } else {
                Self::MD5
            }
        }
    }
    /// Length we need to decode/encode this type of Checksum
    pub fn len(&self) -> usize {
        match self {
            Self::XOR8 => 1,
            Self::XOR16 => 2,
            Self::XOR32 => 4,
            Self::MD5 => 16,
        }
    }
    /// Helper to decode checksum value as unsigned 128,
    /// which covers all scenarios
    pub fn decode(&self, slice: &[u8], len: usize, big_endian: bool) -> u128 {
        if len == 1 {
            slice[0] as u128
        } else if len == 2 {
            let val_u16 = if big_endian {
                u16::from_be_bytes([slice[0], slice[1]])
            } else {
                u16::from_le_bytes([slice[0], slice[1]])
            };
            val_u16 as u128
        } else if len == 4 {
            let val_u32 = if big_endian {
                u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]])
            } else {
                u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])
            };
            val_u32 as u128
        } else {
            panic!("md5");
        }
    }
    /// Calculates expected Checksum for this msg
    pub fn calc(&self, bytes: &[u8], mlen: usize) -> u128 {
        match self {
            Self::XOR8 => Self::xor8_calc(bytes, mlen),
            Self::XOR16 => Self::xor16_calc(bytes),
            Self::XOR32 => Self::xor32_calc(bytes),
            Self::MD5 => Self::md5_calc(bytes),
        }
    }
    /// Calculates expected Checksum using XOR8 algorithm
    fn xor8_calc(bytes: &[u8], size: usize) -> u128 {
        let mut xor = bytes[0];
        for i in 1..size {
            xor ^= bytes[i];
        }
        xor as u128
    }
    /// Calculates expected Checksum using XOR16 algorithm
    fn xor16_calc(bytes: &[u8]) -> u128 {
        0
    }
    /// Calculates expected Checksum using XO32 algorithm
    fn xor32_calc(bytes: &[u8]) -> u128 {
        0
    }
    /// Calculates expected Checksum using MD5 algorithm
    fn md5_calc(bytes: &[u8]) -> u128 {
        let mut hasher = Md5::new();
        hasher.update(bytes);
        let md5 = hasher.finalize();
        u128::from_le_bytes(md5.into())
    }
}

#[cfg(test)]
mod test {
    use super::Checksum;
    #[test]
    fn test_xor8() {
        let buf = [0, 1, 2, 3, 4];
        assert_eq!(Checksum::XOR8.calc(&buf, 5), 4);
    }
}
