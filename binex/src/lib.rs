#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use thiserror::Error;

mod decoder;
mod message;
mod stream;

pub(crate) mod utils;

pub mod prelude {
    pub use crate::{
        decoder::Decoder,
        message::{
            EphemerisFrame, GALEphemeris, GLOEphemeris, GPSEphemeris, GPSRaw, Message, Meta,
            MonumentGeoMetadata, MonumentGeoRecord, Record, SBASEphemeris, TimeResolution,
        },
        stream::{ClosedSourceElement, Provider, StreamElement},
        ClosedSourceMeta, Error,
    };
    // re-export
    pub use hifitime::Epoch;
}

use crate::message::Meta;
use crate::stream::Provider;

/// [ClosedSourceMeta] helps identify a closed source message we cannot interprate.
#[derive(Debug, Copy, Clone)]
pub struct ClosedSourceMeta {
    /// Message ID "as is"
    pub mid: u32,
    /// Message length (total payload) "as is"
    pub mlen: usize,
    /// Size of chunk.
    /// This library is designed to support all open source messages that are short.
    /// Yet a BINEX (prototype) message may span 2^27 bytes.
    pub size: usize,
    /// [Meta] data that follows the open source protocol.
    pub open_meta: Meta,
    /// [Provider] of this message. Only this organization may fully decode this message.
    pub provider: Provider,
    // payload offset in buffer
    offset: usize,
}

#[derive(Debug)]
pub enum Error {
    /// Not enough bytes available to continue decoding process
    NotEnoughBytes,
    /// I/O error
    IoError,
    /// Missing SYNC byte
    NoSyncByte,
    // InvalidStartofStream,
    /// Library limitation: reversed streams are not supported
    ReversedStream,
    /// Library limitation: little endian streams are not verified yet
    LittleEndianStream,
    /// Library limitation: enhanced CRC is not supported yet
    EnhancedCrc,
    /// Found an unsupported timescale that we cannot interprate.
    NonSupportedTimescale,
    /// Found unknown message ID
    UnknownMessage,
    /// Error while attempting to interprate UTF-8 (invalid ASCII)
    Utf8Error,
    /// Message is missing CRC field and cannot be verified
    MissingCRC,
    /// Message corrupt: received CRC does not match expected CRC
    CorrupctBadCRC,
    /// Incomplete message: need more data to complete
    IncompleteMessage(usize),
    /// Library limitation: not all open source Messages supported yet
    NonSupportedMesssage(usize),
    /// Library limtation: should never happen, because this library
    /// will be designed to parse all open source [Message]s.
    /// This may happen as either we're still in development (bad internal design)
    /// or for format that we still do not support (temporarily "ok")
    TooLargeInternalLimitation,
    /// Found closed source message
    ClosedSourceMessage(ClosedSourceMeta),
}
