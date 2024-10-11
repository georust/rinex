mod mid; // message ID
mod record; // Record: message content
mod time;

pub use record::Record;
pub use time::TimeResolution;

pub(crate) use mid::MessageID;

use crate::{constants::Constants, message::record::MonumentMarkerRecord, utils::Utils, Error};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
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
        Self {
            record,
            time_res,
            reversed,
            big_endian,
            enhanced_crc,
            mid: record.message_id(),
        }
    }
    /// [Message] encoding attempt into buffer.
    /// [Self] must fit in preallocated size.
    /// Returns total encoded size, which is equal to the message size (in bytes).
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
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

        // 1. locate SYNC byte
        if let Some(offset) = Self::locate(Constants::FWDSYNC_BE_STANDARD_CRC, buf) {
            big_endian = true;
            sync_off = offset;
        } else if let Some(offset) = Self::locate(Constants::FWDSYNC_LE_STANDARD_CRC, buf) {
            sync_off = offset;
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

        // TODO: unhandled cases
        if reversed {
            return Err(Error::ReversedStream);
        }
        if enhanced_crc {
            return Err(Error::EnhancedCrc);
        }

        // 1* make sure we have enough bytes
        if buf.len() - sync_off < 8 {
            // can't parse MID
            // can't parse MLEN
            // not consistent with minimal payload len:
            // -> ABORT
            return Err(Error::NotEnoughBytes);
        }

        // 2. parse MID
        let mid =
            MessageID::from(Self::decode_bnxi(&buf[sync_off..sync_off + 1], big_endian) as u8);

        // 3. parse MLEN
        let mlen = Self::decode_bnxi(&buf[sync_off + 1..sync_off + 8 + 1], big_endian);

        // 4. parse RECORD
        let record = match mid {
            MessageID::SiteMonumentMarker => {
                let rec = MonumentMarkerRecord::decode(0, time_res, &buf[sync_off + 8..])?;
                Record::new_monument_marker(rec)
            },
            MessageID::Unknown => {
                return Err(Error::UnknownMessage);
            },
        };

        Ok(Self {
            mid,
            record,
            reversed,
            time_res,
            big_endian,
            enhanced_crc,
        })
    }
    /// Tries to locate desired byte within buffer
    fn locate(to_find: u8, buf: &[u8]) -> Option<usize> {
        buf.iter().position(|b| *b == to_find)
    }
    /// Evaluates CRC for [Self]
    pub(crate) fn eval_crc(&self) -> u32 {
        0
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

            // println!("bytes: {:?}", bytes);
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
    use super::Message;
    #[test]
    fn big_endian_bnxi() {
        for (bytes, expected) in [
            ([0x7f, 0x81, 0x7f].to_vec(), 0x7f),
            ([0x7f, 0x81, 0x7f, 0x7f, 0x7f].to_vec(), 0x7f),
            ([0x83, 0x7a].to_vec(), 0x7a03),
        ] {
            assert_eq!(Message::decode_bnxi(&bytes, true), expected);
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
            assert_eq!(Message::decode_bnxi(&bytes, false), expected);
        }
    }
}
