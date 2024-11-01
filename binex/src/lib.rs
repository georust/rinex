#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use thiserror::Error;

mod decoder;
mod message;
mod stream;

pub(crate) mod constants;
pub(crate) mod utils;

pub mod prelude {
    pub use crate::{
        decoder::Decoder,
        message::{
            EphemerisFrame, GALEphemeris, GLOEphemeris, GPSEphemeris, GPSRaw, Message,
            MonumentGeoMetadata, MonumentGeoRecord, Record, SBASEphemeris, TimeResolution,
        },
        stream::{ClosedSourceElement, Provider, StreamElement},
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
    #[error("no sync byte")]
    NoSyncByte,
    #[error("reversed streams are not supported yet")]
    ReversedStream,
    #[error("little endian encoded streams not supported yet")]
    LittleEndianStream,
    #[error("enhanced crc is not supported yet")]
    EnhancedCrc,
    #[error("non supported timescale")]
    NonSupportedTimescale,
    // #[error("unknown message")]
    // UnknownMessage,
    #[error("unknown record field id")]
    UnknownRecordFieldId,
    #[error("utf8 error")]
    Utf8Error,
    #[error("missing crc bytes")]
    MissingCRC,
    #[error("received invalid crc")]
    BadCRC,
    #[error("incomplete: need more data")]
    IncompleteMessage(usize),
    #[error("non supported message: library limitation")]
    NonSupportedMesssage(usize),
    #[error("message too large: library limitation")]
    // This message should never happen: library is to be designed
    // to support largest open source (fully disclosed) message frame
    TooLargeInternalLimitation,
}
