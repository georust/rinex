//! Checksum calculator
use lazy_static::lazy_static;
use md5::{Digest, Md5};

/// Checksum caculator
#[derive(Debug, Copy, Clone)]
pub enum Checksum {
    XOR8,
    XOR16,
    XOR32,
    MD5,
}

lazy_static! {
    static ref CRC16_TABLE: [u16; 256] = [
        0x0000, 0x8161, 0x83A3, 0x02C2, 0x8627, 0x0746, 0x0584, 0x84E5, 0x8D2F, 0x0C4E, 0x0E8C,
        0x8FED, 0x0B08, 0x8A69, 0x88AB, 0x09CA, 0x9B3F, 0x1A5E, 0x189C, 0x99FD, 0x1D18, 0x9C79,
        0x9EBB, 0x1FDA, 0x1610, 0x9771, 0x95B3, 0x14D2, 0x9037, 0x1156, 0x1394, 0x92F5, 0xB71F,
        0x367E, 0x34BC, 0xB5DD, 0x3138, 0xB059, 0xB29B, 0x33FA, 0x3A30, 0xBB51, 0xB993, 0x38F2,
        0xBC17, 0x3D76, 0x3FB4, 0xBED5, 0x2C20, 0xAD41, 0xAF83, 0x2EE2, 0xAA07, 0x2B66, 0x29A4,
        0xA8C5, 0xA10F, 0x206E, 0x22AC, 0xA3CD, 0x2728, 0xA649, 0xA48B, 0x25EA, 0xEF5F, 0x6E3E,
        0x6CFC, 0xED9D, 0x6978, 0xE819, 0xEADB, 0x6BBA, 0x6270, 0xE311, 0xE1D3, 0x60B2, 0xE457,
        0x6536, 0x67F4, 0xE695, 0x7460, 0xF501, 0xF7C3, 0x76A2, 0xF247, 0x7326, 0x71E4, 0xF085,
        0xF94F, 0x782E, 0x7AEC, 0xFB8D, 0x7F68, 0xFE09, 0xFCCB, 0x7DAA, 0x5840, 0xD921, 0xDBE3,
        0x5A82, 0xDE67, 0x5F06, 0x5DC4, 0xDCA5, 0xD56F, 0x540E, 0x56CC, 0xD7AD, 0x5348, 0xD229,
        0xD0EB, 0x518A, 0xC37F, 0x421E, 0x40DC, 0xC1BD, 0x4558, 0xC439, 0xC6FB, 0x479A, 0x4E50,
        0xCF31, 0xCDF3, 0x4C92, 0xC877, 0x4916, 0x4BD4, 0xCAB5, 0x5FDF, 0xDEBE, 0xDC7C, 0x5D1D,
        0xD9F8, 0x5899, 0x5A5B, 0xDB3A, 0xD2F0, 0x5391, 0x5153, 0xD032, 0x54D7, 0xD5B6, 0xD774,
        0x5615, 0xC4E0, 0x4581, 0x4743, 0xC622, 0x42C7, 0xC3A6, 0xC164, 0x4005, 0x49CF, 0xC8AE,
        0xCA6C, 0x4B0D, 0xCFE8, 0x4E89, 0x4C4B, 0xCD2A, 0xE8C0, 0x69A1, 0x6B63, 0xEA02, 0x6EE7,
        0xEF86, 0xED44, 0x6C25, 0x65EF, 0xE48E, 0xE64C, 0x672D, 0xE3C8, 0x62A9, 0x606B, 0xE10A,
        0x73FF, 0xF29E, 0xF05C, 0x713D, 0xF5D8, 0x74B9, 0x767B, 0xF71A, 0xFED0, 0x7FB1, 0x7D73,
        0xFC12, 0x78F7, 0xF996, 0xFB54, 0x7A35, 0xB080, 0x31E1, 0x3323, 0xB242, 0x36A7, 0xB7C6,
        0xB504, 0x3465, 0x3DAF, 0xBCCE, 0xBE0C, 0x3F6D, 0xBB88, 0x3AE9, 0x382B, 0xB94A, 0x2BBF,
        0xAADE, 0xA81C, 0x297D, 0xAD98, 0x2CF9, 0x2E3B, 0xAF5A, 0xA690, 0x27F1, 0x2533, 0xA452,
        0x20B7, 0xA1D6, 0xA314, 0x2275, 0x079F, 0x86FE, 0x843C, 0x055D, 0x81B8, 0x00D9, 0x021B,
        0x837A, 0x8AB0, 0x0BD1, 0x0913, 0x8872, 0x0C97, 0x8DF6, 0x8F34, 0x0E55, 0x9CA0, 0x1DC1,
        0x1F03, 0x9E62, 0x1A87, 0x9BE6, 0x9924, 0x1845, 0x118F, 0x90EE, 0x922C, 0x134D, 0x97A8,
        0x16C9, 0x140B, 0x956A,
    ];
    static ref CRC32_TABLE: [u32; 256] = [
        0x0000, 0x4C11B7, 0x98236E, 0xD432D9, 0x13046DC, 0x17C576B, 0x1A865B2, 0x1E47405,
        0x2608DB8, 0x22C9C0F, 0x2F8AED6, 0x2B4BF61, 0x350CB64, 0x31CDAD3, 0x3C8E80A, 0x384F9BD,
        0x4C11B70, 0x48D0AC7, 0x459381E, 0x41529A9, 0x5F15DAC, 0x5BD4C1B, 0x5697EC2, 0x5256F75,
        0x6A196C8, 0x6ED877F, 0x639B5A6, 0x675A411, 0x791D014, 0x7DDC1A3, 0x709F37A, 0x745E2CD,
        0x98236E0, 0x9CE2757, 0x91A158E, 0x9560439, 0x8B2703C, 0x8FE618B, 0x82A5352, 0x86642E5,
        0xBE2BB58, 0xBAEAAEF, 0xB7A9836, 0xB368981, 0xAD2FD84, 0xA9EEC33, 0xA4ADEEA, 0xA06CF5D,
        0xD432D90, 0xD0F3C27, 0xDDB0EFE, 0xD971F49, 0xC736B4C, 0xC3F7AFB, 0xCEB4822, 0xCA75995,
        0xF23A028, 0xF6FB19F, 0xFBB8346, 0xFF792F1, 0xE13E6F4, 0xE5FF743, 0xE8BC59A, 0xEC7D42D,
        0x13046DC0, 0x13487C77, 0x139C4EAE, 0x13D05F19, 0x12342B1C, 0x12783AAB, 0x12AC0872,
        0x12E019C5, 0x1164E078, 0x1128F1CF, 0x11FCC316, 0x11B0D2A1, 0x1054A6A4, 0x1018B713,
        0x10CC85CA, 0x1080947D, 0x17C576B0, 0x17896707, 0x175D55DE, 0x17114469, 0x16F5306C,
        0x16B921DB, 0x166D1302, 0x162102B5, 0x15A5FB08, 0x15E9EABF, 0x153DD866, 0x1571C9D1,
        0x1495BDD4, 0x14D9AC63, 0x140D9EBA, 0x14418F0D, 0x1A865B20, 0x1ACA4A97, 0x1A1E784E,
        0x1A5269F9, 0x1BB61DFC, 0x1BFA0C4B, 0x1B2E3E92, 0x1B622F25, 0x18E6D698, 0x18AAC72F,
        0x187EF5F6, 0x1832E441, 0x19D69044, 0x199A81F3, 0x194EB32A, 0x1902A29D, 0x1E474050,
        0x1E0B51E7, 0x1EDF633E, 0x1E937289, 0x1F77068C, 0x1F3B173B, 0x1FEF25E2, 0x1FA33455,
        0x1C27CDE8, 0x1C6BDC5F, 0x1CBFEE86, 0x1CF3FF31, 0x1D178B34, 0x1D5B9A83, 0x1D8FA85A,
        0x1DC3B9ED, 0x2608DB80, 0x2644CA37, 0x2690F8EE, 0x26DCE959, 0x27389D5C, 0x27748CEB,
        0x27A0BE32, 0x27ECAF85, 0x24685638, 0x2424478F, 0x24F07556, 0x24BC64E1, 0x255810E4,
        0x25140153, 0x25C0338A, 0x258C223D, 0x22C9C0F0, 0x2285D147, 0x2251E39E, 0x221DF229,
        0x23F9862C, 0x23B5979B, 0x2361A542, 0x232DB4F5, 0x20A94D48, 0x20E55CFF, 0x20316E26,
        0x207D7F91, 0x21990B94, 0x21D51A23, 0x210128FA, 0x214D394D, 0x2F8AED60, 0x2FC6FCD7,
        0x2F12CE0E, 0x2F5EDFB9, 0x2EBAABBC, 0x2EF6BA0B, 0x2E2288D2, 0x2E6E9965, 0x2DEA60D8,
        0x2DA6716F, 0x2D7243B6, 0x2D3E5201, 0x2CDA2604, 0x2C9637B3, 0x2C42056A, 0x2C0E14DD,
        0x2B4BF610, 0x2B07E7A7, 0x2BD3D57E, 0x2B9FC4C9, 0x2A7BB0CC, 0x2A37A17B, 0x2AE393A2,
        0x2AAF8215, 0x292B7BA8, 0x29676A1F, 0x29B358C6, 0x29FF4971, 0x281B3D74, 0x28572CC3,
        0x28831E1A, 0x28CF0FAD, 0x350CB640, 0x3540A7F7, 0x3594952E, 0x35D88499, 0x343CF09C,
        0x3470E12B, 0x34A4D3F2, 0x34E8C245, 0x376C3BF8, 0x37202A4F, 0x37F41896, 0x37B80921,
        0x365C7D24, 0x36106C93, 0x36C45E4A, 0x36884FFD, 0x31CDAD30, 0x3181BC87, 0x31558E5E,
        0x31199FE9, 0x30FDEBEC, 0x30B1FA5B, 0x3065C882, 0x3029D935, 0x33AD2088, 0x33E1313F,
        0x333503E6, 0x33791251, 0x329D6654, 0x32D177E3, 0x3205453A, 0x3249548D, 0x3C8E80A0,
        0x3CC29117, 0x3C16A3CE, 0x3C5AB279, 0x3DBEC67C, 0x3DF2D7CB, 0x3D26E512, 0x3D6AF4A5,
        0x3EEE0D18, 0x3EA21CAF, 0x3E762E76, 0x3E3A3FC1, 0x3FDE4BC4, 0x3F925A73, 0x3F4668AA,
        0x3F0A791D, 0x384F9BD0, 0x38038A67, 0x38D7B8BE, 0x389BA909, 0x397FDD0C, 0x3933CCBB,
        0x39E7FE62, 0x39ABEFD5, 0x3A2F1668, 0x3A6307DF, 0x3AB73506, 0x3AFB24B1, 0x3B1F50B4,
        0x3B534103, 0x3B8773DA, 0x3BCB626D,
    ];
}

impl Checksum {
    /// Determines [Checksum] for this message length
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
            unimplemented!("md5");
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
        let mut crc = 0xffff_u16;
        for byte in bytes.iter() {
            // let tmp = (crc >> 8) ^ *byte as u16;
            // crc = (crc << 8) ^ CRC16_TABLE[tmp as usize];

            let tmp = (*byte as u16) ^ crc;
            crc >>= 8;
            crc ^= CRC16_TABLE[(tmp as usize) % 256];
        }
        crc as u128
    }
    /// Calculates expected Checksum using XO32 algorithm
    fn xor32_calc(bytes: &[u8]) -> u128 {
        let mut crc = 0xffffffff_u32;
        for byte in bytes.iter() {
            let tmp = (*byte as u32) ^ crc;
            crc >>= 8;
            crc ^= CRC32_TABLE[(tmp as usize) % 256];
        }
        crc as u128
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

        let buf = [
            0x00, 0x1f, 0x01, 0x39, 0x87, 0x20, 0x00, 0x00, 0x00, 0x17, 0x42, 0x49, 0x4e, 0x45,
            0x58, 0x20, 0x53, 0x74, 0x72, 0x65, 0x61, 0x6d, 0x20, 0x52, 0x65, 0x73, 0x74, 0x61,
            0x72, 0x74, 0x65, 0x64, 0x21,
        ];
        assert_eq!(Checksum::XOR8.calc(&buf, buf.len()), 0x84);
    }

    #[test]
    fn test_xor16() {
        // 0xe2, 0x01=MID, 0x81=MLEN,
        let buf = [
            0x01, 0x81, 0x00, 0x01, 0x1d, 0x07, 0xf6, 0x00, 0x03, 0xd8, 0x72, 0x00, 0x03, 0xf4,
            0x80, 0x31, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0xac,
            0xdc, 0x00, 0x00, 0xb8, 0x38, 0xc3, 0x00, 0x00, 0x00, 0x00, 0x20, 0x30, 0xd5, 0x5c,
            0x00, 0xbf, 0xf8, 0x96, 0x4c, 0x65, 0x6e, 0xda, 0x41, 0x3f, 0x6d, 0x97, 0xd5, 0xd0,
            0x00, 0x00, 0x00, 0x40, 0xb4, 0x21, 0xa2, 0x39, 0x40, 0x00, 0x00, 0x32, 0x20, 0x00,
            0x00, 0x43, 0x44, 0xf8, 0x00, 0xb3, 0x18, 0x00, 0x00, 0x42, 0x78, 0x60, 0x00, 0x36,
            0x49, 0xa0, 0x00, 0x37, 0x16, 0x60, 0x00, 0x40, 0x02, 0xa8, 0x2c, 0x0b, 0x2a, 0x18,
            0x0c, 0xc0, 0x08, 0x23, 0xb8, 0x97, 0xbd, 0xf9, 0x99, 0x3f, 0xee, 0x23, 0x55, 0xce,
            0x2e, 0x11, 0x70, 0xb1, 0x31, 0xa4, 0x00, 0xad, 0xac, 0x00, 0x00, 0x41, 0xa0, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x04,
        ];

        assert_eq!(Checksum::XOR16.calc(&buf, buf.len()), 0x7d49);
    }
}
