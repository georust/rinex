//! Monument Geodetic marker specific frames

use crate::{
    message::{record::monument::FieldID, Message},
    Error,
};

use log::error;

#[derive(Debug, Clone, PartialEq)]
pub enum MonumentGeoFrame {
    /// Comment
    Comment(String),
}

impl MonumentGeoFrame {
    /// Returns total length (bytewise) required to fully encode [Self].
    /// Use this to fulfill [Self::encode] requirements.
    pub(crate) fn encoding_size(&self) -> usize {
        match self {
            Self::Comment(s) => {
                let s_len = s.len();
                s_len + Message::bnxi_encoding_size(s_len as u32)
            },
        }
    }

    /// Returns expected [FieldID] for [Self]
    pub(crate) fn to_field_id(&self) -> FieldID {
        match self {
            Self::Comment(_) => FieldID::Comment,
        }
    }

    /// [MonumentGeoFrame] decoding attempt from given [FieldID]
    pub(crate) fn decode(fid: FieldID, big_endian: bool, buf: &[u8]) -> Result<Self, Error> {
        println!("bnx00-monument_geo: {:?}", buf);

        match fid {
            FieldID::Comment => {
                if buf.len() < 1 {
                    return Err(Error::NotEnoughBytes); // can't parse BNXI
                }

                let (s_len, off) = Message::decode_bnxi(&buf, big_endian);

                println!("strlen={}", s_len);

                if buf.len() - off < s_len as usize {
                    return Err(Error::NotEnoughBytes); // can't parse entire string
                }

                match std::str::from_utf8(&buf[off..]) {
                    Ok(s) => Ok(Self::Comment(s.to_string())),
                    Err(e) => {
                        error!("bnx00(geo)-comment: utf8 error {}", e);
                        Err(Error::Utf8Error)
                    },
                }
            },
            _ => Err(Error::UnknownRecordFieldId),
        }
    }

    /// Encodes [Self] into buffer, returns encoded size (total bytes).
    /// [Self] must fit in preallocated buffer.
    pub fn encode(&self, big_endian: bool, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.encoding_size();
        if buf.len() < size {
            return Err(Error::NotEnoughBytes);
        }

        match self {
            Self::Comment(s) => {
                // s_len as 1-4 Byte
                let s_len = s.len();
                let size = Message::encode_bnxi(s_len as u32, big_endian, buf)?;
                buf[size..size + s_len].clone_from_slice(s.as_bytes()); // utf8 encoding
            },
        }

        Ok(size)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn geo_comments() {
        let comment = "hello geo".to_string();
        let s_len = comment.len();
        let frame = MonumentGeoFrame::Comment(comment);
        assert_eq!(frame.encoding_size(), s_len + 1);

        let big_endian = true;
        let mut buf = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let size = frame.encode(big_endian, &mut buf).unwrap();

        assert_eq!(size, frame.encoding_size());
        assert_eq!(
            buf,
            [9, 104, 101, 108, 108, 111, 32, 103, 101, 111, 0, 0, 0]
        )
    }
}
