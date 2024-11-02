mod checksum;
mod mid; // message ID
mod record; // Record: message content
mod time; // Epoch encoding/decoding // checksum calc.

pub use record::{
    EphemerisFrame, GALEphemeris, GLOEphemeris, GPSEphemeris, GPSRaw, MonumentGeoMetadata,
    MonumentGeoRecord, Record, SBASEphemeris,
};

pub use time::TimeResolution;

pub(crate) use mid::MessageID;

use checksum::Checksum;

use crate::{constants::Constants, stream::Provider, ClosedSourceMeta, Error};

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
    pub record: Record,
    /// Time Resolution in use
    time_res: TimeResolution,
}

impl Message {
    /// Creates a new [Message] ready to be encoded.
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

    /// Returns total size required to encode this [Message].
    /// Use this to fulfill [Self::encode] requirements.
    pub fn encoding_size(&self) -> usize {
        let mut total = 1; // SYNC

        let mid = self.record.to_message_id() as u32;
        total += Self::bnxi_encoding_size(mid);

        let mlen = self.record.encoding_size();
        total += Self::bnxi_encoding_size(mlen as u32);

        total += self.record.encoding_size();

        let ck = Checksum::from_len(mlen, self.enhanced_crc);
        total += ck.len();

        total
    }

    /// [Message] decoding attempt from buffered content.
    /// Buffer must contain sync byte and the following frame must match
    /// the specification if an open source BINEX [Message].
    /// For closed source [Message]s, we return [Error::ClosedSourceMessage]
    /// with header information.
    pub fn decode(buf: &[u8]) -> Result<Self, Error> {
        let sync_off;
        let buf_len = buf.len();
        let mut big_endian = true;
        let mut reversed = false;
        let mut enhanced_crc = false;
        let time_res = TimeResolution::QuarterSecond;

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

        // <!> TODO: non supported cases <!>
        //    * Rev Streams are not supported
        //    * Only basic CRC is managed
        //    * Little Endianness not tested yet!
        if reversed {
            return Err(Error::ReversedStream);
        }
        if enhanced_crc {
            return Err(Error::EnhancedCrc);
        }
        if !big_endian {
            return Err(Error::LittleEndianStream);
        }

        // make sure we can parse up to 4 byte MID
        if buf_len - sync_off < 4 {
            return Err(Error::NotEnoughBytes);
        }

        let mut ptr = sync_off + 1;

        // 2. parse MID
        let (bnxi, mid_1_4) = Self::decode_bnxi(&buf[ptr..], big_endian);
        let mid = MessageID::from(bnxi);
        ptr += mid_1_4;

        // make sure we can parse up to 4 byte MLEN
        if buf_len - ptr < 4 {
            return Err(Error::NotEnoughBytes);
        }

        // 3. parse MLEN
        let (mlen, mlen_1_4) = Self::decode_bnxi(&buf[ptr..], big_endian);
        let mlen = mlen as usize;
        // println!("mlen={}", mlen);

        if buf_len - ptr < mlen {
            // buffer does not contain complete message!
            return Err(Error::IncompleteMessage(mlen));
        }

        ptr += mlen_1_4;

        // 4. parse RECORD
        let record = match mid {
            MessageID::SiteMonumentMarker => {
                let rec =
                    MonumentGeoRecord::decode(mlen as usize, time_res, big_endian, &buf[ptr..])?;
                Record::new_monument_geo(rec)
            },
            MessageID::Ephemeris => {
                let fr = EphemerisFrame::decode(big_endian, &buf[ptr..])?;
                Record::new_ephemeris_frame(fr)
            },
            _ => {
                // check whether this message is undisclosed or not
                if let Some(provider) = Provider::match_any(mid.into()) {
                    return Err(Error::ClosedSourceMessage(ClosedSourceMeta {
                        mlen,
                        provider,
                        offset: ptr,
                        mid: mid.into(),
                    }));
                } else {
                    // println!("found unsupported msg id={:?}", id);
                    return Err(Error::NonSupportedMesssage(mlen));
                }
            },
        };

        // 5. CRC
        let checksum = Checksum::from_len(mlen, enhanced_crc);
        let ck_len = checksum.len();

        if ptr + mlen + ck_len > buf_len {
            return Err(Error::MissingCRC);
        }

        // decode
        let ck = checksum.decode(&buf[ptr + mlen..], ck_len, big_endian);

        // verify
        let expected = checksum.calc(&buf[sync_off + 1..], mlen + mid_1_4 + mlen_1_4);

        if expected != ck {
            Err(Error::CorrupctBadCRC)
        } else {
            Ok(Self {
                mid,
                record,
                reversed,
                time_res,
                big_endian,
                enhanced_crc,
            })
        }
    }

    /// [Message] encoding attempt into buffer.
    /// [Self] must fit in preallocated size.
    /// Returns total encoded size, which is equal to the message size (in bytes).
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, Error> {
        let total = self.encoding_size();
        if buf.len() < total {
            return Err(Error::NotEnoughBytes);
        }

        // Encode SYNC byte
        buf[0] = self.sync_byte();
        let mut ptr = 1;

        // Encode MID
        let mid = self.record.to_message_id() as u32;
        let mid_1_4 = Self::encode_bnxi(mid, self.big_endian, &mut buf[ptr..])?;
        ptr += mid_1_4;

        // Encode MLEN
        let mlen = self.record.encoding_size();
        let mlen_1_4 = Self::encode_bnxi(mlen as u32, self.big_endian, &mut buf[ptr..])?;
        ptr += mlen_1_4;

        // Encode message
        match &self.record {
            Record::EphemerisFrame(fr) => {
                ptr += fr.encode(self.big_endian, &mut buf[ptr..])?;
            },
            Record::MonumentGeo(geo) => {
                ptr += geo.encode(self.big_endian, &mut buf[ptr..])?;
            },
        }

        // encode CRC
        let ck = Checksum::from_len(mlen, self.enhanced_crc);
        let ck_len = ck.len();
        let crc_u128 = ck.calc(&buf[1..], mlen + mid_1_4 + mlen_1_4);

        if ck_len == 1 {
            buf[ptr] = crc_u128 as u8;
        } else if ck_len == 2 {
            let crc_bytes = if self.big_endian {
                (crc_u128 as u16).to_be_bytes()
            } else {
                (crc_u128 as u16).to_le_bytes()
            };

            for i in 0..ck_len {
                buf[ptr + i] = crc_bytes[i];
            }
        } else if ck_len == 4 {
            let crc_bytes = if self.big_endian {
                (crc_u128 as u32).to_be_bytes()
            } else {
                (crc_u128 as u32).to_le_bytes()
            };
            for i in 0..ck_len {
                buf[ptr + i] = crc_bytes[i];
            }
        } else {
            let crc_bytes = if self.big_endian {
                crc_u128.to_be_bytes()
            } else {
                crc_u128.to_le_bytes()
            };
            for i in 0..ck_len {
                buf[ptr + i] = crc_bytes[i];
            }
        }

        Ok(ptr + ck_len)
    }

    /// Returns the SYNC byte we expect for [Self]
    pub(crate) fn sync_byte(&self) -> u8 {
        if self.reversed {
            if self.big_endian {
                if self.enhanced_crc {
                    Constants::REVSYNC_BE_ENHANCED_CRC
                } else {
                    Constants::REVSYNC_BE_STANDARD_CRC
                }
            } else {
                if self.enhanced_crc {
                    Constants::REVSYNC_LE_ENHANCED_CRC
                } else {
                    Constants::REVSYNC_LE_STANDARD_CRC
                }
            }
        } else {
            if self.big_endian {
                if self.enhanced_crc {
                    Constants::FWDSYNC_BE_ENHANCED_CRC
                } else {
                    Constants::FWDSYNC_BE_STANDARD_CRC
                }
            } else {
                if self.enhanced_crc {
                    Constants::FWDSYNC_LE_ENHANCED_CRC
                } else {
                    Constants::FWDSYNC_LE_STANDARD_CRC
                }
            }
        }
    }

    /// Tries to locate desired byte within buffer
    fn locate(to_find: u8, buf: &[u8]) -> Option<usize> {
        buf.iter().position(|b| *b == to_find)
    }

    /// Number of bytes to encode U32 using the 1-4 BNXI algorithm.
    pub(crate) const fn bnxi_encoding_size(val: u32) -> usize {
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

    /// Decodes 1-4 BNXI encoded unsigned U32 integer with selected endianness,
    /// according to [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html/#ubnxi_details].
    /// ## Outputs
    ///    * u32: decoded U32 integer
    ///     * usize: number of bytes consumed in this process
    ///       ie., last byte contributing to the BNXI encoding.
    ///       The next byte is the following content.
    pub(crate) fn decode_bnxi(buf: &[u8], big_endian: bool) -> (u32, usize) {
        let min_size = buf.len().min(4);

        // handle bad op
        if min_size == 0 {
            return (0, 0);
        }

        // single byte case
        if buf[0] & Constants::BNXI_KEEP_GOING_MASK == 0 {
            let val32 = buf[0] as u32;
            return (val32 & 0x7f, 1);
        }

        // multi byte case
        let (val, size) = if buf[1] & Constants::BNXI_KEEP_GOING_MASK == 0 {
            let mut val;

            let (byte0, byte1) = if big_endian {
                (buf[0], buf[1])
            } else {
                (buf[1], buf[0])
            };

            val = (byte0 & Constants::BNXI_BYTE_MASK) as u32;
            val <<= 7;
            val |= byte1 as u32;

            (val, 2)
        } else if buf[2] & Constants::BNXI_KEEP_GOING_MASK == 0 {
            let mut val;

            let (byte0, byte1, byte2) = if big_endian {
                (buf[0], buf[1], buf[2])
            } else {
                (buf[2], buf[1], buf[0])
            };

            val = (byte0 & Constants::BNXI_BYTE_MASK) as u32;
            val <<= 8;

            val |= (byte1 & Constants::BNXI_BYTE_MASK) as u32;
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

            val = (byte0 & Constants::BNXI_BYTE_MASK) as u32;
            val <<= 8;

            val |= (byte1 & Constants::BNXI_BYTE_MASK) as u32;
            val <<= 8;

            val |= (byte2 & Constants::BNXI_BYTE_MASK) as u32;
            val <<= 7;

            val |= byte3 as u32;
            (val, 4)
        };

        (val, size)
    }

    /// U32 to BNXI encoder according to [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html/#ubnxi_details].
    /// Encodes into given buffer, returns encoding size.
    /// Will fail if buffer is too small.
    pub(crate) fn encode_bnxi(val: u32, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
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
}

#[cfg(test)]
mod test {
    use super::Message;
    use crate::message::{EphemerisFrame, GPSRaw, MonumentGeoMetadata, MonumentGeoRecord, Record};
    use crate::message::{GALEphemeris, GPSEphemeris, TimeResolution};
    use crate::prelude::Epoch;
    use crate::{constants::Constants, Error};

    #[test]
    fn big_endian_bnxi() {
        let buf = [0];
        let (decoded, size) = Message::decode_bnxi(&buf, true);
        assert_eq!(size, 1);
        assert_eq!(decoded, 0);

        let mut encoded = [0; 4];
        let size = Message::encode_bnxi(decoded, true, &mut encoded).unwrap();

        assert_eq!(size, 1);
        assert_eq!(encoded, [0, 0, 0, 0]);

        let buf = [0, 0, 0, 0];
        let (decoded, size) = Message::decode_bnxi(&buf, true);
        assert_eq!(size, 1);
        assert_eq!(decoded, 0);

        let mut encoded = [0; 4];
        let size = Message::encode_bnxi(decoded, true, &mut encoded).unwrap();

        assert_eq!(size, 1);
        assert_eq!(encoded, [0, 0, 0, 0]);

        let buf = [1, 0, 0, 0];
        let (decoded, size) = Message::decode_bnxi(&buf, true);
        assert_eq!(size, 1);
        assert_eq!(decoded, 1);

        let mut encoded = [0; 4];
        let size = Message::encode_bnxi(decoded, true, &mut encoded).unwrap();

        assert_eq!(size, 1);
        assert_eq!(encoded, [1, 0, 0, 0]);

        let buf = [2, 0, 0, 0];
        let (decoded, size) = Message::decode_bnxi(&buf, true);
        assert_eq!(size, 1);
        assert_eq!(decoded, 2);

        let mut encoded = [0; 4];
        let size = Message::encode_bnxi(decoded, true, &mut encoded).unwrap();

        assert_eq!(size, 1);
        assert_eq!(encoded, [2, 0, 0, 0]);

        let buf = [127, 0, 0, 0];
        let (decoded, size) = Message::decode_bnxi(&buf, true);
        assert_eq!(size, 1);
        assert_eq!(decoded, 127);

        let mut encoded = [0; 4];
        let size = Message::encode_bnxi(decoded, true, &mut encoded).unwrap();

        assert_eq!(size, 1);
        assert_eq!(encoded, [127, 0, 0, 0]);

        let buf = [129, 0, 0, 0];
        let (decoded, size) = Message::decode_bnxi(&buf, true);
        assert_eq!(size, 2);
        assert_eq!(decoded, 128);

        let mut encoded = [0; 4];
        let size = Message::encode_bnxi(decoded, true, &mut encoded).unwrap();

        assert_eq!(size, 2);
        assert_eq!(encoded, buf);

        let buf = [0x83, 0x7a, 0, 0];
        let (decoded, size) = Message::decode_bnxi(&buf, true);
        assert_eq!(size, 2);
        assert_eq!(decoded, 0x1fa);

        let mut encoded = [0; 4];
        let size = Message::encode_bnxi(decoded, true, &mut encoded).unwrap();

        assert_eq!(size, 2);
        assert_eq!(encoded, buf);

        let buf = [0x83, 0x83, 0x7a, 0];
        let (decoded, size) = Message::decode_bnxi(&buf, true);
        assert_eq!(size, 3);
        assert_eq!(decoded, 0x181fa);

        let mut encoded = [0; 4];
        let size = Message::encode_bnxi(decoded, true, &mut encoded).unwrap();

        assert_eq!(size, 3);
        assert_eq!(encoded, buf);

        let buf = [0x83, 0x83, 0x83, 0x7a];
        let (decoded, size) = Message::decode_bnxi(&buf, true);
        assert_eq!(size, 4);
        assert_eq!(decoded, 0x18181fa);

        let mut encoded = [0; 4];
        let size = Message::encode_bnxi(decoded, true, &mut encoded).unwrap();

        assert_eq!(size, 4);
        assert_eq!(encoded, buf);
    }

    #[test]
    fn bigend_bnxi_1() {
        for val in [0, 1, 10, 120, 122, 127] {
            let mut buf = [0; 1];
            let size = Message::encode_bnxi(val, true, &mut buf).unwrap();

            assert_eq!(size, 1);
            assert_eq!(buf[0], val as u8);

            let mut buf = [0; 4];

            let size = Message::encode_bnxi(val, true, &mut buf).unwrap();

            assert_eq!(size, 1);

            assert_eq!(buf[0], val as u8);
            assert_eq!(buf[1], 0);
            assert_eq!(buf[2], 0);
            assert_eq!(buf[3], 0);

            let (decoded, size) = Message::decode_bnxi(&buf, true);
            assert_eq!(size, 1);
            assert_eq!(decoded, val);
        }
    }

    #[test]
    fn decode_no_sync_byte() {
        let buf = [0, 0, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::NoSyncByte) => {},
            Err(e) => panic!("returned unexpected error: {:?}", e),
            _ => panic!("should have paniced"),
        }
        let buf = [0, 0, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::NoSyncByte) => {},
            Err(e) => panic!("returned unexpected error: {:?}", e),
            _ => panic!("should have paniced"),
        }
    }

    #[test]
    fn decode_fwd_enhancedcrc_stream() {
        let buf = [Constants::FWDSYNC_BE_ENHANCED_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::EnhancedCrc) => {},
            Err(e) => panic!("returned unexpected error: {:?}", e),
            _ => panic!("should have paniced"),
        }
    }

    #[test]
    fn decode_fwd_le_stream() {
        let buf = [Constants::FWDSYNC_LE_STANDARD_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::LittleEndianStream) => {},
            Err(e) => panic!("returned unexpected error: {:?}", e),
            _ => panic!("should have paniced"),
        }
    }

    #[test]
    fn decode_reversed_stream() {
        let buf = [Constants::REVSYNC_BE_STANDARD_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::ReversedStream) => {},
            Err(e) => panic!("returned unexpected error: {:?}", e),
            _ => panic!("should have paniced"),
        }
        let buf = [Constants::REVSYNC_BE_ENHANCED_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::ReversedStream) => {},
            Err(e) => panic!("returned unexpected error: {:?}", e),
            _ => panic!("should have paniced"),
        }
        let buf = [Constants::REVSYNC_LE_STANDARD_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::ReversedStream) => {},
            Err(e) => panic!("returned unexpected error: {:?}", e),
            _ => panic!("should have paniced"),
        }
        let buf = [Constants::REVSYNC_LE_ENHANCED_CRC, 0, 0, 0];
        match Message::decode(&buf) {
            Err(Error::ReversedStream) => {},
            Err(e) => panic!("returned unexpected error: {:?}", e),
            _ => panic!("should have paniced"),
        }
    }

    #[test]
    fn test_monument_geo() {
        let big_endian = true;
        let enhanced_crc = false;
        let reversed = false;

        let mut geo = MonumentGeoRecord::default().with_comment("simple");

        geo.epoch = Epoch::from_gpst_seconds(1.0);
        geo.meta = MonumentGeoMetadata::RNX2BIN;

        let geo_len = geo.encoding_size();
        let record = Record::new_monument_geo(geo);

        let msg = Message::new(
            big_endian,
            TimeResolution::QuarterSecond,
            enhanced_crc,
            reversed,
            record,
        );

        // SYNC + MID(1) +FID + MLEN + CRC(8)
        assert_eq!(msg.encoding_size(), 1 + 1 + 1 + geo_len + 1);

        let mut encoded = [0; 256];
        msg.encode(&mut encoded).unwrap();

        assert_eq!(encoded[17], 3);

        // parse back
        let parsed = Message::decode(&encoded).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_gps_raw() {
        let big_endian = true;
        let enhanced_crc = false;
        let reversed = false;

        let gps_raw = EphemerisFrame::GPSRaw(GPSRaw::default());
        let gps_raw_len = gps_raw.encoding_size();
        let record = Record::new_ephemeris_frame(gps_raw);

        let msg = Message::new(
            big_endian,
            TimeResolution::QuarterSecond,
            enhanced_crc,
            reversed,
            record,
        );

        // SYNC + MID(1) + MLEN(1) + RLEN + CRC(1)
        assert_eq!(msg.encoding_size(), 1 + 1 + 1 + gps_raw_len + 1);

        let mut encoded = [0; 256];
        msg.encode(&mut encoded).unwrap();

        assert_eq!(encoded[78 + 1 + 1 + 1], 0);

        // parse back
        let parsed = Message::decode(&encoded).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_gps_eph() {
        let big_endian = true;
        let enhanced_crc = false;
        let reversed = false;

        let gps_eph = EphemerisFrame::GPS(GPSEphemeris::default());
        let gps_eph_len = gps_eph.encoding_size();
        let record = Record::new_ephemeris_frame(gps_eph);

        assert_eq!(gps_eph_len, 129);

        let msg = Message::new(
            big_endian,
            TimeResolution::QuarterSecond,
            enhanced_crc,
            reversed,
            record,
        );

        // SYNC + MID(1) + MLEN(2) + RLEN + CRC(2)
        assert_eq!(msg.encoding_size(), 1 + 1 + 2 + gps_eph_len + 2);

        let mut encoded = [0; 256];
        msg.encode(&mut encoded).unwrap();

        // parse back
        let parsed = Message::decode(&encoded).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_gal_eph() {
        let big_endian = true;
        let enhanced_crc = false;
        let reversed = false;

        let eph = EphemerisFrame::GAL(GALEphemeris::default());
        let eph_len = eph.encoding_size();
        let record = Record::new_ephemeris_frame(eph);

        assert_eq!(eph_len, 129);

        let msg = Message::new(
            big_endian,
            TimeResolution::QuarterSecond,
            enhanced_crc,
            reversed,
            record,
        );

        // SYNC + MID(1) + MLEN(2) + RLEN + CRC(2)
        assert_eq!(msg.encoding_size(), 1 + 1 + 2 + eph_len + 2);

        let mut encoded = [0; 256];
        msg.encode(&mut encoded).unwrap();

        // parse back
        let parsed = Message::decode(&encoded).unwrap();
        assert_eq!(parsed, msg);
    }
}
