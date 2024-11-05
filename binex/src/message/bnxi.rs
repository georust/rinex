use crate::prelude::{Error, Message};

impl Message {
    /// Number of bytes to encode U32 using the 1-4 BNXI algorithm.
    pub const fn bnxi_encoding_size(val: u32) -> usize {
        if val < 128 {
            1
        } else if val < 16384 {
            2
        } else if val < 2097152 {
            3
        } else {
            4
        }
    }
    /// U32 to BNXI encoder according to [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html/#ubnxi_details].
    /// Encodes into given buffer, returns encoding size.
    /// Will fail if buffer is too small.
    pub fn encode_bnxi(val: u32, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = Self::bnxi_encoding_size(val);
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        // single byte case
        if size == 1 {
            buf[0] = (val as u8) & 0x7f;
            return Ok(1);
        }

        // multi byte case
        let mut val32 = (val & 0xffffff80) << 1;
        val32 |= val & 0xff;

        if size == 2 {
            val32 |= 0x8000;
            val32 &= 0xff7f;

            if big_endian {
                buf[0] = ((val32 & 0xff00) >> 8) as u8;
                buf[1] = val32 as u8;
            } else {
                buf[1] = ((val32 & 0xff00) >> 8) as u8;
                buf[0] = val32 as u8;
            }
        } else if size == 3 {
            val32 |= 0x808000;
            val32 &= 0xffff7f;

            if big_endian {
                buf[0] = ((val32 & 0xffff00) >> 16) as u8;
                buf[1] = ((val32 & 0xff00) >> 8) as u8;
                buf[2] = val32 as u8;
            } else {
                buf[2] = ((val32 & 0xffff00) >> 16) as u8;
                buf[1] = ((val32 & 0xff00) >> 8) as u8;
                buf[0] = val32 as u8;
            }
        } else {
            val32 |= 0x80808000;
            val32 &= 0xffffff7f;

            if big_endian {
                buf[0] = ((val32 & 0xffffff00) >> 24) as u8;
                buf[1] = ((val32 & 0xffff00) >> 16) as u8;
                buf[2] = ((val32 & 0xff00) >> 8) as u8;
                buf[3] = val32 as u8;
            } else {
                buf[3] = ((val32 & 0xffffff00) >> 24) as u8;
                buf[2] = ((val32 & 0xffff00) >> 16) as u8;
                buf[1] = ((val32 & 0xff00) >> 8) as u8;
                buf[0] = val32 as u8;
            }
        }

        Ok(size)
    }
    /// Decodes 1-4 BNXI encoded unsigned U32 integer with selected endianness,
    /// according to [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html/#ubnxi_details].
    /// ## Outputs
    ///    * u32: decoded U32 integer
    ///     * usize: number of bytes consumed in this process
    ///       ie., last byte contributing to the BNXI encoding.
    ///       The next byte is the following content.
    pub fn decode_bnxi(buf: &[u8], big_endian: bool) -> (u32, usize) {
        let min_size = buf.len().min(4);

        // handle bad op
        if min_size == 0 {
            return (0, 0);
        }

        // single byte case
        if buf[0] & Self::BNXI_KEEP_GOING_MASK == 0 {
            let val32 = buf[0] as u32;
            return (val32 & 0x7f, 1);
        }

        // multi byte case
        let (val, size) = if buf[1] & Self::BNXI_KEEP_GOING_MASK == 0 {
            let mut val;

            let (byte0, byte1) = if big_endian {
                (buf[0], buf[1])
            } else {
                (buf[1], buf[0])
            };

            val = (byte0 & Self::BNXI_BYTE_MASK) as u32;
            val <<= 7;
            val |= byte1 as u32;

            (val, 2)
        } else if buf[2] & Self::BNXI_KEEP_GOING_MASK == 0 {
            let mut val;

            let (byte0, byte1, byte2) = if big_endian {
                (buf[0], buf[1], buf[2])
            } else {
                (buf[2], buf[1], buf[0])
            };

            val = (byte0 & Self::BNXI_BYTE_MASK) as u32;
            val <<= 8;

            val |= (byte1 & Self::BNXI_BYTE_MASK) as u32;
            val <<= 7;

            val |= byte2 as u32;
            (val, 3)
        } else {
            let mut val;

            let (byte0, byte1, byte2, byte3) = if big_endian {
                (buf[0], buf[1], buf[2], buf[3])
            } else {
                (buf[3], buf[2], buf[1], buf[0])
            };

            val = (byte0 & Self::BNXI_BYTE_MASK) as u32;
            val <<= 8;

            val |= (byte1 & Self::BNXI_BYTE_MASK) as u32;
            val <<= 8;

            val |= (byte2 & Self::BNXI_BYTE_MASK) as u32;
            val <<= 7;

            val |= byte3 as u32;
            (val, 4)
        };

        (val, size)
    }
}
