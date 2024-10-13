//! Monument / Geodetic marker frames

use crate::{
    message::time::{decode_gpst_epoch, encode_epoch, TimeResolution},
    Error,
};

use hifitime::{Epoch, TimeScale};

mod fid;
mod frame;
mod src;

// private
use fid::FieldID;

// public
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

impl Iterator for MonumentGeoRecord {
    type Item = MonumentGeoFrame;
    fn next(&mut self) -> Option<Self::Item> {
        self.frames.iter().next().cloned()
    }
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

    /// Creates new empty [MonumentGeoRecord], which is not suitable for encoding yet.
    /// Use other method to customize it!
    /// ```
    /// use binex::prelude::{
    ///     Epoch,
    ///     Error,
    ///     MonumentGeoRecord,
    ///     MonumentGeoMetadata,
    /// };
    ///     
    /// let t = Epoch::from_gpst_seconds(60.0 + 0.75);
    ///
    /// let record = MonumentGeoRecord::new(t, MonumentGeoMetadata::RNX2BIN)
    ///     .with_comment("A B C")
    ///     // read comments carefuly. For example, unlike `comments`
    ///     // we're not allowed to define more than one geophysical_info.
    ///     // Otherwise, to frame to be forged will not respect the standards.
    ///     .with_geophysical_info("Eurasian plate")
    ///     .with_climatic_info("Rain!");
    ///
    /// let mut buf = [0; 8];
    /// match record.encode(true, &mut buf) {
    ///     Ok(_) => {
    ///         panic!("encoding should have failed!");
    ///     },
    ///     Err(Error::NotEnoughBytes) => {
    ///         // This frame does not fit in this pre allocated buffer.
    ///         // You should always tie your allocations to .encoding_size() !
    ///     },
    ///     Err(e) => {
    ///         panic!("{} error should not have happened!", e);
    ///     },
    /// }
    ///
    /// let mut buf = [0; 64];
    /// let _ = record.encode(true, &mut buf)
    ///     .unwrap();
    /// ```
    pub fn new(epoch: Epoch, meta: MonumentGeoMetadata) -> Self {
        Self {
            epoch,
            source_meta: meta,
            frames: Vec::with_capacity(8),
        }
    }

    /// [Self] decoding attempt from buffered content.
    /// ## Inputs
    ///    - mlen: message length in bytes
    ///    - time_res: [TimeResolution]
    ///    - big_endian: endianness
    ///    - buf: buffered content
    /// ## Outputs
    ///    - Ok: [Self]
    ///    - Err: [Error]
    pub fn decode(
        mlen: usize,
        time_res: TimeResolution,
        big_endian: bool,
        buf: &[u8],
    ) -> Result<Self, Error> {
        if mlen < Self::MIN_SIZE {
            // does not look good
            return Err(Error::NotEnoughBytes);
        }

        // decode timestamp
        let epoch = decode_gpst_epoch(big_endian, time_res, &buf)?;

        // decode source meta
        let source_meta = MonumentGeoMetadata::from(buf[5]);

        // parse inner frames (= subrecords)
        let mut ptr = 6;
        let mut frames = Vec::<MonumentGeoFrame>::with_capacity(8);

        // this method tolerates badly duplicated subrecords
        while ptr < mlen {
            // decode field id
            match MonumentGeoFrame::decode(big_endian, &buf[ptr..]) {
                Ok(fr) => {
                    ptr += fr.encoding_size();
                    frames.push(fr);
                },
                Err(_) => {
                    if ptr == 6 {
                        // did not parse a single record: incorrect message
                        return Err(Error::NotEnoughBytes);
                    } else {
                        break; // parsed all records
                    }
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
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        // encode tstamp
        let mut ptr = encode_epoch(self.epoch.to_time_scale(TimeScale::GPST), big_endian, buf)?;

        // encode source meta
        buf[ptr] = self.source_meta.into();
        ptr += 1;

        // encode subrecords
        for fr in self.frames.iter() {
            let offs = fr.encode(big_endian, &mut buf[ptr..])?;
            ptr += offs;
        }

        Ok(self.encoding_size())
    }

    /// Returns total length (bytewise) required to fully encode [Self].
    /// Use this to fulfill [Self::encode] requirements.
    pub(crate) fn encoding_size(&self) -> usize {
        let mut size = 6; // tstamp + source_meta
        for fr in self.frames.iter() {
            size += fr.encoding_size(); // content
        }
        size
    }

    /// Add one [MonumentGeoFrame::Comment] to [MonumentGeoRecord].
    /// You can add as many as needed.
    pub fn with_comment(&self, comment: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(MonumentGeoFrame::Comment(comment.to_string()));
        s
    }

    /// Attach readable Geophysical information (like local tectonic plate)
    /// to this [MonumentGeoRecord]. You can only add one per dataset
    /// otherwise, the message will not respect the standard definitions.
    pub fn with_geophysical_info(&self, info: &str) -> Self {
        let mut s = self.clone();
        s.frames
            .push(MonumentGeoFrame::Geophysical(info.to_string()));
        s
    }

    /// Provide Climatic or Meteorological information (local to reference site).
    /// You can only add one per dataset otherwise,
    /// the message will not respect the standard definitions.
    pub fn with_climatic_info(&self, info: &str) -> Self {
        let mut s = self.clone();
        s.frames.push(MonumentGeoFrame::Climatic(info.to_string()));
        s
    }

    /// Define a readable UserID to attach to this [MonumentGeoRecord] dataset.
    /// You can only add one per dataset otherwise,
    /// the message will not respect the standard definitions.
    pub fn with_user_id(&self, userid: &str) -> Self {
        let mut s = self.clone();
        s.frames.push(MonumentGeoFrame::Comment(userid.to_string()));
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn monument_marker_bnx00_error() {
        let buf = [0, 0, 0, 0];
        let time_res = TimeResolution::QuarterSecond;
        let monument = MonumentGeoRecord::decode(4, time_res, true, &buf);
        assert!(monument.is_err());
    }

    #[test]
    fn monument_geo_comments_decoding() {
        let mlen = 17;
        let big_endian = true;
        let time_res = TimeResolution::QuarterSecond;

        let buf = [
            0, 0, 1, 1, 41, 2, 0, 9, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8,
            ' ' as u8, 'G' as u8, 'E' as u8, 'O' as u8,
        ];

        match MonumentGeoRecord::decode(mlen, time_res, big_endian, &buf) {
            Ok(monument) => {
                assert_eq!(
                    monument.epoch,
                    Epoch::from_gpst_seconds(256.0 * 60.0 + 60.0 + 10.25)
                );
                assert_eq!(monument.source_meta, MonumentGeoMetadata::IGS);
                assert_eq!(monument.frames.len(), 1);
                assert_eq!(
                    monument.frames[0],
                    MonumentGeoFrame::Comment("Hello GEO".to_string())
                );

                // test mirror op
                let mut target = [0, 0, 0, 0, 0, 0, 0, 0];
                assert!(monument.encode(big_endian, &mut target).is_err());

                // test mirror op
                let mut target = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
                match monument.encode(big_endian, &mut target) {
                    Err(e) => panic!("{} should have passed", e),
                    Ok(_) => {
                        assert_eq!(target, buf,);
                    },
                }
            },
            Err(e) => panic!("decoding error: {}", e),
        }
    }

    #[test]
    fn monument_geo_double_comments_decoding() {
        let t = Epoch::from_gpst_seconds(60.0 + 0.75);

        let record = MonumentGeoRecord::new(t, MonumentGeoMetadata::RNX2BIN)
            .with_comment("A B C")
            .with_comment("D E F");

        let mut buf = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        match record.encode(true, &mut buf) {
            Ok(_) => panic!("should have panic'ed!"),
            Err(Error::NotEnoughBytes) => {},
            Err(e) => panic!("invalid error: {}", e),
        }

        let mut buf = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        match record.encode(true, &mut buf) {
            Err(e) => panic!("{} should have passed!", e),
            Ok(size) => {
                assert_eq!(size, 20);
                assert_eq!(
                    buf,
                    [0, 0, 0, 1, 3, 1, 0, 5, 65, 32, 66, 32, 67, 0, 5, 68, 32, 69, 32, 70, 0]
                );
            },
        }

        let geo = MonumentGeoRecord::new(t, MonumentGeoMetadata::IGS)
            .with_comment("Hello")
            .with_climatic_info("Clim");

        let mut buf = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        match geo.encode(true, &mut buf) {
            Err(e) => panic!("{} should have passed!", e),
            Ok(size) => {
                assert_eq!(size, 19);
                assert_eq!(
                    buf,
                    [
                        0, 0, 0, 1, 3, 2, 0, 5, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8,
                        'o' as u8, 14, 4, 'C' as u8, 'l' as u8, 'i' as u8, 'm' as u8, 0, 0
                    ]
                );
            },
        }
    }
}
