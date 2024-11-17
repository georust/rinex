//! RINEX compression / decompression module
use thiserror::Error;

// TODO
// Improve Result<> IoError/Error relation ship
// the current User API .read().error()
// Will trigger a string comparison on every single .read() acces veritication

mod compressor;
mod crinex;
mod decompressor;
mod numdiff;
mod obs;
mod textdiff;

pub use compressor::Compressor;
pub use crinex::CRINEX;

pub use decompressor::{Decompressor, DecompressorExpert};

pub(crate) use numdiff::NumDiff;
pub(crate) use obs::ObsDiff;
pub(crate) use textdiff::TextDiff;

use thiserror::Error as ErrorTrait;

/// Hatanaka dedicated Errors
#[derive(Debug, ErrorTrait)]
pub enum Error {
    /// Buffer too small to accept incoming data
    #[error("buffer overflow")]
    BufferOverflow,
    /// Forwarded Epoch description does not look good: Invalid RINEX!
    #[error("invalid epoch format")]
    EpochFormat,
    /// Forwarded content is not consisten with CRINEX V1
    #[error("invalid v1 format")]
    BadV1Format,
    /// Forwarded content is not consisten with CRINEX V3
    #[error("invalid v3 format")]
    BadV3Format,
    /// [SV] identification error: bad relationship between
    /// either:
    ///   - recovered Epoch description (in decompression scheme)
    ///   and parsing process
    ///   - invalid data being forwared and/or incompatibility
    ///   with previously formwared Header
    #[error("sv parsing error")]
    SVParsing,
}
