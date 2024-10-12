mod mid; // message ID
mod record; // Record: message content
mod time; // Epoch encoding/decoding

pub use record::{MonumentGeoMetadata, MonumentGeoRecord, Record};

pub use time::TimeResolution;

pub(crate) use mid::MessageID;

use crate::{constants::Constants, utils::Utils, Error};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Message {
    /// Endianness used when encoding current message,
    /// defined by SYNC byte
    pub big_endian: bool,
    /// MID byte stored as [MessageID]
    mid: MessageID,
    /// True when using enhanced CRC
    pub enhanced_crc: bool,
    /// True when reversible stream
    pub reversed: bool,
    /// [Record]
    record: Record,
    /// Time Resolution in use
    time_res: TimeResolution,
}

impl Message {
    /// Creates new [Message] from given specs, ready to encode.
    pub fn new(
        big_endian: bool,
        time_res: TimeResolution,
        enhanced_crc: bool,
        reversed: bool,
        record: Record,
    ) -> Self {
        let mid = record.to_message_id();
        Self {
            mid,
            record,
            time_res,
            reversed,
            big_endian,
            enhanced_crc,
        }
    }

    /// Decoding attempt from buffered content.
    pub fn decode(buf: &[u8]) -> Result<Self, Error> {
        let mut size = 0usize;
        let mut sync_off = 0usize;
        let mut big_endian = true;
        let mut reversed = false;
        let mut enhanced_crc = false;
        let mut mid = MessageID::default();
        let mut time_res = TimeResolution::QuarterSecond;

        // print!("decoding {:?}", buf);

        // 1. locate SYNC byte
        if let Some(offset) = Self::locate(Constants::FWDSYNC_BE_STANDARD_CRC, buf) {
            big_endian = true;
            sync_off = offset;
        } else if let Some(offset) = Self::locate(Constants::FWDSYNC_LE_STANDARD_CRC, buf) {
            sync_off = offset;
            big_endian = false;
        } else if let Some(offset) = Self::locate(Constants::FWDSYNC_BE_ENHANCED_CRC, buf) {
            big_endian = true;
            enhanced_crc = true;
            sync_off = offset;
        } else if let Some(offset) = Self::locate(Constants::FWDSYNC_LE_ENHANCED_CRC, buf) {
            enhanced_crc = true;
            sync_off = offset;
        } else if let Some(offset) = Self::locate(Constants::REVSYNC_LE_STANDARD_CRC, buf) {
            reversed = true;
            sync_off = offset;
        } else if let Some(offset) = Self::locate(Constants::REVSYNC_BE_STANDARD_CRC, buf) {
            reversed = true;
            big_endian = true;
            sync_off = offset;
        } else if let Some(offset) = Self::locate(Constants::REVSYNC_BE_ENHANCED_CRC, buf) {
            reversed = true;
            big_endian = true;
            enhanced_crc = true;
            sync_off = offset;
        } else if let Some(offset) = Self::locate(Constants::REVSYNC_LE_ENHANCED_CRC, buf) {
            reversed = true;
            enhanced_crc = true;
            sync_off = offset;
        } else {
            // no SYNC byte found
            return Err(Error::NoSyncByte);
        }

        // TODO: non supported cases (WIP)
        if reversed {
            return Err(Error::ReversedStream);
        }
        if enhanced_crc {
            return Err(Error::EnhancedCrc);
        }
        if !big_endian {
            return Err(Error::LittleEndianStream);
        }

        // 1* make sure we have enough bytes
        if buf.len() - sync_off < 4 {
            // TODO improve this test ? Depends on following 1-4 byte
            return Err(Error::NotEnoughBytes);
        }

        let mut ptr = sync_off + 1;

        // 2. parse MID
        let (bnxi, size) = Self::decode_bnxi(&buf[ptr..], big_endian);
        let mid = MessageID::from(bnxi);
        println!("mid={:?}", mid);
        ptr += size;

        // 3. parse MLEN
        let (mlen, size) = Self::decode_bnxi(&buf[ptr..], big_endian);

        if buf.len() < mlen as usize {
            // buffered content does not match stream specs
            return Err(Error::NotEnoughBytes);
        }

        println!("mlen={:?}", mlen);
        ptr += size;

        // 4. parse RECORD
        let record = match mid {
            MessageID::SiteMonumentMarker => {
                let rec = MonumentGeoRecord::decode(
                    (mlen - 1) as usize, // CRC!
                    time_res,
                    big_endian,
                    &buf[ptr..],
                )?;
                Record::new_monument_geo(rec)
            },
            MessageID::Unknown => {
                println!("id=0xffffffff");
                return Err(Error::UnknownMessage);
            },
            id => {
                println!("found unsupported msg id={:?}", id);
                return Err(Error::UnknownMessage);
            },
        };

        // 5. CRC verification

        Ok(Self {
            mid,
            record,
            reversed,
            time_res,
            big_endian,
            enhanced_crc,
        })
    }

    /// [Message] encoding attempt into buffer.
    /// [Self] must fit in preallocated size.
    /// Returns total encoded size, which is equal to the message size (in bytes).
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        Ok(0)
    }

    /// Tries to locate desired byte within buffer
    fn locate(to_find: u8, buf: &[u8]) -> Option<usize> {
        buf.iter().position(|b| *b == to_find)
    }

    /// Returns total length (bytewise) to fully encode [Self].
    /// Use this to fullfill [Self::encode] requirements.
    pub fn encoding_size(&self) -> usize {
        // MID 1-4 byte encoding
        let mid = Self::bnxi_encoding_size(self.mid.into());

        // actual record length
        let rlen = self.record.encoding_size();

        // MLEN 1-4 byte encoding
        let mlen = Self::bnxi_encoding_size(rlen as u32);

        mid + mlen + rlen + 1 + 1 // +SYNC BYTE + CRC
    }

    // /// Evaluates CRC for [Self]
    // pub(crate) fn eval_crc(&self) -> u32 {
    //     0
    // }

    /// Decodes BNXI encoded unsigned U32 integer with selected endianness,
    /// according to [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html/#ubnxi_details].
    /// ## Outputs
    ///    * u32: decoded U32 integer
    ///     * usize: number of bytes consumed in this process
    ///       ie., last byte contributing to the BNXI encoding.
    ///       The next byte is the following content.
    pub(crate) fn decode_bnxi(buf: &[u8], big_endian: bool) -> (u32, usize) {
        let mut last_preserved = 0;

        // handles invalid case
        if buf.len() == 1 {
            if buf[0] & Constants::BNXI_KEEP_GOING_MASK > 0 {
                return (0, 0);
            }
        }

        for i in 0..buf.len() {
            if i < 3 {
                if buf[i] & Constants::BNXI_KEEP_GOING_MASK == 0 {
                    last_preserved = i;
                    break;
                }
            } else {
                last_preserved = i;
            }
        }

        // apply mask
        let masked = buf
            .iter()
            .enumerate()
            .map(|(j, b)| {
                if j == 3 {
                    *b
                } else {
                    *b & Constants::BNXI_BYTE_MASK
                }
            })
            .collect::<Vec<_>>();

        let mut ret = 0_u32;

        // interprate as desired
        if big_endian {
            for i in 0..=last_preserved {
                ret += (masked[i] as u32) << (8 * i);
            }
        } else {
            for i in 0..=last_preserved {
                ret += (masked[i] as u32) << ((4 - i) * 8);
            }
        }

        (ret, last_preserved + 1)
    }

    /// Number of bytes to encode U32 unsigned integer
    /// following the 1-4 BNXI encoding algorithm
    pub(crate) fn bnxi_encoding_size(val: u32) -> usize {
        let bytes = (val as f64).log2().ceil() as usize / 8 + 1;
        Utils::min_usize(bytes, 4)
    }

    /// U32 to BNXI encoder according to [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html/#ubnxi_details].
    /// Encodes into given buffer, returns encoding size.
    /// Will fail if buffer is too small.
    pub(crate) fn encode_bnxi(val: u32, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let bytes = Self::bnxi_encoding_size(val);
        if buf.len() < bytes {
            return Err(Error::NotEnoughBytes);
        }

        for i in 0..bytes {
            if big_endian {
                buf[i] = (val >> (8 * i)) as u8;
                if i < 3 {
                    buf[i] &= Constants::BNXI_BYTE_MASK;
                }
            } else {
                buf[bytes - 1 - i] = (val >> (8 * i)) as u8;
                if i < 3 {
                    buf[bytes - 1 - i] &= Constants::BNXI_BYTE_MASK;
                }
            }

            if i > 0 {
                if big_endian {
                    buf[i - 1] |= Constants::BNXI_KEEP_GOING_MASK;
                } else {
                    buf[bytes - 1 - i - 1] |= Constants::BNXI_KEEP_GOING_MASK;
                }
            }
        }

        return Ok(bytes);
    }
}

#[cfg(test)]
mod test {
    use super::Message;
    use crate::{constants::Constants, Error};
    #[test]
    fn big_endian_bnxi_1() {
        let bytes = [0x7a];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 1);
        assert_eq!(val, 0x7a);

        // test mirror op
        let mut buf = [0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 1);
        assert_eq!(buf, [0x7a]);

        let mut buf = [0, 0, 0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 1);
        assert_eq!(buf, [0x7a, 0, 0, 0]);

        // invalid case
        let bytes = [0x81];
        let (_, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 0);
    }

    #[test]
    fn big_endian_bnxi_2() {
        let bytes = [0x7a, 0x81];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 1);
        assert_eq!(val, 0x7a);

        // test mirror op
        let mut buf = [0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 1);
        assert_eq!(buf, [0x7a, 0]);

        let bytes = [0x83, 0x7a];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 2);
        assert_eq!(val, 0x7a03);

        // test mirror op
        let mut buf = [0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 2);
        assert_eq!(buf, [0x83, 0x7a]);
    }

    #[test]
    fn big_endian_bnxi_3() {
        let bytes = [0x83, 0x84, 0x7a];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 3);
        assert_eq!(val, 0x7a0403);

        let bytes = [0x83, 0x84, 0x7a, 0];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 3);
        assert_eq!(val, 0x7a0403);

        let bytes = [0x83, 0x84, 0x7a, 0, 0];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 3);
        assert_eq!(val, 0x7a0403);

        // test mirror op
        let mut buf = [0, 0, 0, 0, 0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 3);
        assert_eq!(buf, [0x83, 0x84, 0x7a, 0, 0, 0]);
    }

    #[test]
    fn big_endian_bnxi_4() {
        let bytes = [0x7f, 0x81, 0x7f, 0xab];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 1);
        assert_eq!(val, 0x7f);

        // test mirror
        let mut buf = [0, 0, 0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 1);
        assert_eq!(buf, [0x7f, 0, 0, 0]);

        let bytes = [0x81, 0xaf, 0x7f, 0xab];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 3);
        assert_eq!(val, 0x7f2f01);

        // test mirror
        let mut buf = [0, 0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 3);
        assert_eq!(buf, [0x81, 0xaf, 0x7f]);

        // test mirror
        let mut buf = [0, 0, 0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 3);
        assert_eq!(buf, [0x81, 0xaf, 0x7f, 0]);

        let bytes = [0x81, 0xaf, 0x8f, 1];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 4);
        assert_eq!(val, 0x10f2f01);

        // test mirror
        let mut buf = [0, 0, 0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 4);
        assert_eq!(buf, [0x81, 0xaf, 0x8f, 1]);

        // test mirror
        let mut buf = [0, 0, 0, 0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 4);
        assert_eq!(buf, [0x81, 0xaf, 0x8f, 1, 0]);

        let bytes = [0x81, 0xaf, 0x8f, 0x7f];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 4);
        assert_eq!(val, 0x7f0f2f01);

        // test mirror
        let mut buf = [0, 0, 0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 4);
        assert_eq!(buf, [0x81, 0xaf, 0x8f, 0x7f]);

        let bytes = [0x81, 0xaf, 0x8f, 0x80];
        let (val, size) = Message::decode_bnxi(&bytes, true);
        assert_eq!(size, 4);
        assert_eq!(val, 0x800f2f01);

        // test mirror
        let mut buf = [0, 0, 0, 0];
        let size = Message::encode_bnxi(val, true, &mut buf).unwrap();
        assert_eq!(size, 4);
        assert_eq!(buf, [0x81, 0xaf, 0x8f, 0x80]);
    }

    #[test]
    fn decode_no_sync_byte() {
        let buf = [0, 0, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::NoSyncByte) => {},
            Err(e) => panic!("returned unexpected error: {}", e),
            _ => panic!("should have paniced"),
        }
        let buf = [0, 0, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::NoSyncByte) => {},
            Err(e) => panic!("returned unexpected error: {}", e),
            _ => panic!("should have paniced"),
        }
    }
    #[test]
    fn decode_fwd_enhancedcrc_stream() {
        let buf = [Constants::FWDSYNC_BE_ENHANCED_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::EnhancedCrc) => {},
            Err(e) => panic!("returned unexpected error: {}", e),
            _ => panic!("should have paniced"),
        }
    }
    #[test]
    fn decode_fwd_le_stream() {
        let buf = [Constants::FWDSYNC_LE_STANDARD_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::LittleEndianStream) => {},
            Err(e) => panic!("returned unexpected error: {}", e),
            _ => panic!("should have paniced"),
        }
    }
    #[test]
    fn decode_reversed_stream() {
        let buf = [Constants::REVSYNC_BE_STANDARD_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::ReversedStream) => {},
            Err(e) => panic!("returned unexpected error: {}", e),
            _ => panic!("should have paniced"),
        }
        let buf = [Constants::REVSYNC_BE_ENHANCED_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::ReversedStream) => {},
            Err(e) => panic!("returned unexpected error: {}", e),
            _ => panic!("should have paniced"),
        }
        let buf = [Constants::REVSYNC_LE_STANDARD_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::ReversedStream) => {},
            Err(e) => panic!("returned unexpected error: {}", e),
            _ => panic!("should have paniced"),
        }
        let buf = [Constants::REVSYNC_LE_ENHANCED_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::ReversedStream) => {},
            Err(e) => panic!("returned unexpected error: {}", e),
            _ => panic!("should have paniced"),
        }
    }
}
