#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use thiserror::Error;

mod decoder;
mod message;

pub(crate) mod constants;
pub(crate) mod utils;

pub mod prelude {
    pub use crate::{
        decoder::Decoder,
        message::{Message, MonumentGeoMetadata, MonumentGeoRecord, Record, TimeResolution},
        Error,
    };
    // re-export
    pub use hifitime::Epoch;
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("not enough bytes available")]
    NotEnoughBytes,
    #[error("i/o error")]
    IoError(#[from] std::io::Error),
    #[error("invalid start of stream")]
    InvalidStartofStream,
    #[error("no SYNC byte found")]
    NoSyncByte,
    #[error("reversed streams are not supported yet")]
    ReversedStream,
    #[error("little endian encoded streams not supported yet")]
    LittleEndianStream,
    #[error("enhanced crc is not supported yet")]
    EnhancedCrc,
    #[error("non supported timescale")]
    NonSupportedTimescale,
    #[error("U32 decoding error")]
    U32Decoding,
    #[error("unknown message")]
    UnknownMessage,
    #[error("unknown record field id")]
    UnknownRecordFieldId,
    #[error("utf8 error")]
    Utf8Error,
}
