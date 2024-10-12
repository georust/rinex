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
    /// Returns encoding size for [Self]
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::Comment(s) => s.len(),
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
        match fid {
            FieldID::Comment => {
                if buf.len() < 1 {
                    return Err(Error::NotEnoughBytes); // can't parse BNXI
                }
                let (s_len, off) = Message::decode_bnxi(&buf, big_endian);

                if buf.len() < s_len as usize - off {
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
}
