use crate::constants::Constants;

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
    /// Decodes unsigned U32 integer with selected endianness,
    /// as per [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html/#ubnxi_details]
    pub(crate) fn decode_bnxi(buf: &[u8], big_endian: bool) -> u32 {
        let last_preserved = buf
            .iter()
            .position(|b| *b & Constants::BNXI_KEEP_GOING_MASK == 0);

        if let Some(last_preserved) = last_preserved {
            // apply data mask to all bytes
            let mut bytes = [0_u8; 4];
            let size = Utils::min_usize(last_preserved + 1, 4);

            for i in 0..size {
                bytes[3 - i] = buf[i] & Constants::BNXI_BYTE_MASK;
            }

            println!("bytes: {:?}", bytes);
            // interprate as desired
            if big_endian {
                u32::from_be_bytes(bytes)
            } else {
                u32::from_le_bytes(bytes)
            }
        } else {
            0
        }
    }
}

#[cfg(test)]
mod test {
    use super::Utils;
    #[test]
    fn big_endian_bnxi() {
        for (bytes, expected) in [
            ([0x7f, 0x81, 0x7f].to_vec(), 0x7f),
            ([0x7f, 0x81, 0x7f, 0x7f, 0x7f].to_vec(), 0x7f),
            ([0x83, 0x7a].to_vec(), 0x7a03),
        ] {
            assert_eq!(Utils::decode_bnxi(&bytes, true), expected);
        }
    }
    #[test]
    fn little_endian_bnxi() {
        for (bytes, expected) in [
            ([0x7f, 0x81].to_vec(), 0x17f),
            ([0x7f, 0x81, 0x7f].to_vec(), 0x17f),
            ([0x7f, 0x81, 0x7f, 0x7f, 0x7f].to_vec(), 0x17f),
            ([0x83, 0x7a].to_vec(), 0x37a),
        ] {
            assert_eq!(Utils::decode_bnxi(&bytes, false), expected);
        }
    }
}
