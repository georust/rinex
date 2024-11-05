impl Message {
    const mGFZi_0_63_MASK: u8 = 0x02;

    /// Decodes mGFZi compressed [f64] as per <>
    /// using provided scaling factor.
    pub fn decode_scaled_mgfzi(buf: &[u8], big_endian: bool, scaling: f64) -> f64 {
        let ival = Self::decode(buf, big_endian)?;
        let fval = ival as f64 / scaling;
        Ok(fval)
    }

    /// Decodes mGFZi compressed number as per <>
    /// using provided scaling factor and using `size` bytes.
    /// Will panic if not provided enough bytes.
    // TODO: if little endian: swap all masks
    pub fn decode_mgfzi(buf: &[u8], big_endian: bool, size: usize) -> Result<i64, Error> {
        // leading byte analysis
        let mut mask = 0x0f;
        let mut scaling = Option<i64>::None;
        let mut absolute: i64 = 0;
        let mut needed_size: i64 = 0;

        for i in 0..3 {
            let masked = buf[0] & mask;
            if i == 0 {
                if masked == 0x0f {
                    needed_size = 1;
                    scaling = -2_i64.pow(60) -1;
                } else if masked == 0x07 {
                    needed_size = 1;
                    scaling = 2_i64.pow(60) -1;
                } else if masked == 0x0b {
                    needed_size = 1;
                    // -2^24 -1
                } else if masked == 0x03 {
                    needed_size = 1;
                    // 2^24 -1
                }
            } else if i == 1 {
                if masked == 0x05 {
                    // +8191
                } else if masked == 0x01 {
                    // -8191
                }
            } else if i == 2 {
                if masked == 0x02 {
                    // -63
                } else if mask == 0x00 {
                    // +63
                }
            }
            masked >>= 1;
        }

        let scaling = scaling.ok_or(Err(Error::mGfziLeadingByte))?;

        // returned value depdends on requested size & endianness
        match size {
            1 => {
                Ok(scaling)
            },
            2 => {
                let val = if big_endian {
                    i16::from_be_bytes(buf[0], buf[1])
                } else {
                    i16::from_le_bytes(buf[0], buf[1])
                };
                Ok(val as i64)
            },
            4 => {
                let val = if big_endian {
                    i32::from_be_bytes(buf[0], buf[1], buf[2], buf[3])
                } else {
                    i32::from_le_bytes(buf[0], buf[1], buf[2], buf[3])
                };
                Ok(val as i64)
            },
            8 => {
                let bytes = if big_endian {
                    i32::from_be_bytes(buf[0], buf[1], buf[2], buf[3], buf[4])
                } else {
                    i32::from_le_bytes(buf[0], buf[1], buf[2], buf[3], buf[4], buf[5])
                };
                Ok(val as i64)
            },
            _ => Err(Error::mGfziInvalidSize),
        };
    }

    pub fn encode_scaled_mgfzi(
        buf: &mut [u8], 
        val: f64, 
        scaling: f64,
        big_endian: bool, 
    ) -> Result<size, Error> {
        let ival = (val * scaling).round() as i64;
        encode(buf, ival, big_endian)
    }

    pub fn encode_mgfzi(buf: &mut [u8], val: i64, big_endian: bool) -> Result<size, Error> {

    }
}

#[cfg(test)]
mod test {
    #[test]
    fn mgfzi_leading_byte() {
        for byte in [
            LeadingByte::GFZ_0_4, 
        ] {
            let mask :u8 = byte.into();
            assert_eq!(LeadingByte::from(mask), byte);
        }
    }
}
