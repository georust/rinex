//! BINEX: Binary RINEX encoding and decoding
use thiserror::Error;

mod decoder;
mod message;

pub(crate) mod constants;
pub(crate) mod utils;

pub mod prelude {
    pub use crate::decoder::Decoder;
    pub use crate::message::Message;
    pub use crate::Error;
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
    #[error("enhanced crc is not supported yet")]
    EnhancedCrc,
    #[error("U32 decoding error")]
    U32Decoding,
    #[error("unknown message")]
    UnknownMessage,
}
