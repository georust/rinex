//! RINEX compression / decompression module
use thiserror::Error;

mod compressor;
mod crinex;
mod numdiff;
mod obs;
mod textdiff;

pub use crinex::CRINEX;
pub use numdiff::NumDiff;
pub use obs::ObsDiff;
pub use textdiff::TextDiff;

pub mod decompressor;
pub use decompressor::Decompressor;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("header contains invalid UTF8")]
    HeaderBadUtf8Data,
}
