//! BINEX: Binary RINEX encoding and decoding
use std::io::Read;
use thiserror::Error;

mod parser;

mod frameid;
use frameid::FrameID;

pub(crate) mod constants;

pub mod prelude {
    pub use crate::{parser::Parser, Error, Message};
}

pub struct Message {
    /// Frame ID
    pub(crate) fid: FrameID,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("not enough bytes available")]
    NotEnoughBytes,
    #[error("non supported message")]
    UnknownFrame,
    #[error("i/o error")]
    IoError(#[from] std::io::Error),
    #[error("invalid start of stream")]
    InvalidStartofStream,
}
