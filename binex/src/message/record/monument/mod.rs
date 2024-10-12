//! Monument / Geodetic marker frame
/**
 * Geodetic / Monument marker
 * Does not have subrecord (simple record)
 **/
use crate::{
    message::{
        time::{decode_gpst_epoch, TimeResolution},
        Message,
    },
    Error,
};

use hifitime::Epoch;
use log::{debug, error};

mod builder;
mod fid;
mod frame;
mod src;

// private
use fid::FieldID;

// public
pub use builder::MonumentGeoBuilder;
pub use frame::MonumentGeoFrame;
pub use src::MonumentGeoMetadata;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MonumentGeoRecord {
    /// [Epoch]
    pub epoch: Epoch,
    /// Source of this information
    pub source_meta: MonumentGeoMetadata,
    /// Frames also refered to as Subrecords
    pub frames: Vec<MonumentGeoFrame>,
}

impl MonumentGeoRecord {
    /// 4 byte date uint4       }
    /// 1 byte qsec             } epoch
    /// 1 byte MonumentGeoMetadata
    /// 1 byte FID
    ///     if FID corresponds to a character string
    ///     the next 1-4 BNXI byte represent the number of bytes in the caracter string
    /// follows: FID dependent sequence. See [FieldID].
    const MIN_SIZE: usize = 5 + 1 + 1;

    /// [Self] decoding attempt from buffered content.
    pub fn decode(
        msglen: usize,
        time_res: TimeResolution,
        big_endian: bool,
        buf: &[u8],
    ) -> Result<Self, Error> {
        if buf.len() < Self::MIN_SIZE {
            return Err(Error::NotEnoughBytes);
        }

        // decode timestamp
        let epoch = decode_gpst_epoch(big_endian, time_res, &buf[..5])?;

        // decode source meta
        let source_meta = MonumentGeoMetadata::from(buf[5]);

        // parse inner frames (= subrecords)
        let mut ptr = 6;
        let mut frames = Vec::<MonumentGeoFrame>::with_capacity(8);

        // this method tolerates badly duplicated subrecords
        while ptr < buf.len() {
            // decode field id
            let (bnxi, size) = Message::decode_bnxi(&buf[ptr..], big_endian);
            let fid = FieldID::from(bnxi);
            debug!("monument_geo: fid={:?}", fid);

            match fid {
                FieldID::Unknown => {
                    error!("monument_geo: unknown fid={}", bnxi);
                    ptr += 1;
                    continue;
                },
                fid => match MonumentGeoFrame::decode(fid, big_endian, &buf) {
                    Ok(fr) => {
                        ptr += fr.size();
                        frames.push(fr);
                    },
                    Err(e) => {
                        ptr += 1;
                        continue;
                    },
                },
            }
        }

        Ok(Self {
            epoch,
            frames,
            source_meta,
        })
    }

    /// Encodes [Self] into buffer, returns encoded size (total bytes).
    /// [Self] must fit in preallocated buffer.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn monument_marker_bnx00_error() {
        let buf = [0, 0, 0, 0];
        let mlen = 4;
        let time_res = TimeResolution::QuarterSecond;
        let monument = MonumentGeoRecord::decode(mlen, time_res, true, &buf);
        assert!(monument.is_err());
    }
    #[test]
    fn monument_geo_comments_decoding() {
        let buf = [
            0, 0, 1, 1, 1, 2, 0, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, ' ' as u8,
            'G' as u8, 'E' as u8, 'O' as u8,
        ];
        let mlen = 5;
        let time_res = TimeResolution::QuarterSecond;
        match MonumentGeoRecord::decode(mlen, time_res, true, &buf) {
            Ok(monument) => {
                assert_eq!(
                    monument.epoch,
                    Epoch::from_gpst_seconds(256.0 * 60.0 + 60.0 + 0.25)
                );
                assert_eq!(monument.source_meta, MonumentGeoMetadata::IGS);
            },
            Err(e) => panic!("decoding error: {}", e),
        }
    }
}
