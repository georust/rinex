#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

use thiserror::Error;

#[cfg(feature = "std")]
mod decoder;

mod message;
mod stream;

pub(crate) mod constants;
pub(crate) mod utils;

pub mod prelude {
    #[cfg(feature = "std")]
    pub use crate::decoder::Decoder;
    pub use crate::{
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

#[derive(Debug)]
pub enum Error {
    NotEnoughBytes,
    // #[error("i/o error")]
    // IoError(#[from] std::io::Error),
    InvalidStartofStream,
    NoSyncByte,
    ReversedStream,
    LittleEndianStream,
    EnhancedCrc,
    NonSupportedTimescale,
    /// Non recognized Field ID
    UnknownRecordFieldId,
    /// Bad UTF-8 Data
    Utf8Error,
    /// No CRC provided
    MissingCRC,
    /// Invalid CRC decoded
    BadCRC,
    /// Incomplete frame (missing n bytes)
    IncompleteMessage(usize),
    /// Non supported (unknown message) due to library limitation
    NonSupportedMesssage(usize),
    /// This message should never happen: library is to be designed
    /// to support largest open source (fully disclosed) message frame
    TooLargeInternalLimitation,
}
