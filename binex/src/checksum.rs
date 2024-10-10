use crate::Message;

/// BINEX Checksum Calculator
pub enum Checksum {
    /// For [1 - 2^8-1] message
    /// CRC is 1 byte XOR
    XOR8,
    /// For [2^8, 2^12-1] message
    POLY12,
    /// For [2^12, 2^20-1] message,
    POLY20,
}

impl Checksum {
    const fn binary_mask(&self) -> u32 {
        match self {
            Self::XOR8 => 0xff,
            Self::POLY12 => 0x7ff,
            Self::POLY20 => 0xfff,
        }
    }
    const fn look_up_table(&self) -> &[u8] {
        match self {
            Self::XOR8 => &[0, 0, 0, 0],
            Self::POLY12 => {
                // CRC12 table
                // x^16 + x^12 + x^5 +1
                &[0, 1, 2, 3]
            },
            Self::POLY20 => {
                // CRC16 table
                // x^32 + x^26 + x^23 + x^22 + x^16 + x^12 + x^11 + x^10 + x^8 + x^7 + x^5 + x^4 + x^2 + x^1 +1
                &[0, 1, 2, 3]
            },
        }
    }
    pub fn new(msg: &Message) -> Self {
        if msg.len() < 127 {
            Self::XOR8
        } else if msg.len() < 4096 {
            Self::POLY12
        } else {
            Self::POLY20
        }
    }
    /// Calculates CRC from [Message]
    pub fn calc_from_msg(msg: &Message) -> u32 {
        let bytes = msg.to_bytes();
        Self::calc_from_bytes(bytes)
    }
    /// Macro that verifies this [Message] contains correct CRC
    pub fn crc_ok(msg: &Message) -> bool {
        let crc = msg.crc();
        Self::calc_from_msg(msg) == crc
    }
    /// Calculates CRC from buffer content.
    /// Correct content must be correctly extracted beforehand.
    pub fn calc_from_bytes(raw: &[u8]) -> u32 {
        let size = raw.len();
        if size < 128 {
            // 0-127 bytes: 1 byte checksum XOR all bytes
        } else if size < 4096 {
            // 128-4095 x^16 + x^12 + x^5 + x^0 polynomial
            //let lut = self.look_up_table();
        }
        0
    }
}
