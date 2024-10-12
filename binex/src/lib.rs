#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

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

#[derive(Debug)]
pub enum Error {
    /// Input or Output buffer length mismatch
    NotEnoughBytes,
    /// I/O interface error
    IoError,
    /// Invalid (non supported) SOF.
    /// We're currently limited to Foward BE/LE streams:
    ///    * backwards streams are not supported
    ///    * enhanced CRC is not supported
    InvalidStartofStream,
    /// Missing SYNC byte. Buffer does not contain a SYNC byte.
    NoSyncByte,
    /// We're currently limited to Forward Streams only.
    ReversedStream,
    /// We're currently limited to Big Endianness
    LittleEndianStream,
    /// Only Regular CRC is currently supported.
    EnhancedCrc,
    /// This is not a supported Timescale
    NonSupportedTimescale,
    /// U32 decoding error
    U32Decoding,
    /// Non supported (unknown) message: can't decode or encode.
    UnknownMessage,
    /// Non supported (unknown) record entry: can't decode or encode.
    UnknownRecordFieldId,
    /// String parsing error: invalid utf8 data
    Utf8Error,
}
