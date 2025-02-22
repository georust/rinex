//! Bias solutions specific errors
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// Header line should start with %=
    #[error("missing header delimiter")]
    MissingHeaderDelimiter,
    /// Header line should start with %=BIA
    #[error("invalid bias header")]
    InvalidBiasHeader,
    /// Non recognized file type
    #[error("file type error")]
    FileTypeError(#[from] header::DocumentTypeError),
    #[error("failed to parse datetime")]
    ParseDateTimeError(#[from] ParseDateTimeError),
    #[error("failed to parse `length` field")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse `bias_mode` field")]
    BiasModeError(#[from] BiasModeError),
}
