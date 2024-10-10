//! BINEX: Binary RINEX encoding and decoding
use std::io::Read;
use thiserror::Error;

mod checksum;
mod decoder;
mod frameid;
mod utils;

pub(crate) mod constants;

use frameid::FrameID;

pub mod prelude {
    pub use crate::{decoder::Decoder, Error, Message};
}

pub struct Message {
    /// Frame ID
    pub(crate) fid: FrameID,
}

impl Message {
    /// Message length in Bytes
    pub fn len(&self) -> usize {
        0
    }
    /// Converts self to Byte array
    pub fn to_bytes(&self) -> &[u8] {
        &[0, 1, 2]
    }
    /// Calcualtes CRC for Self
    pub fn crc(&self) -> u32 {
        0
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("not enough bytes available")]
    NotEnoughBytes,
    #[error("format not supported: library limitation")]
    NonSupportedFormat,
    #[error("non supported message")]
    UnknownFrame,
    #[error("i/o error")]
    IoError(#[from] std::io::Error),
    #[error("invalid start of stream")]
    InvalidStartofStream,
}
