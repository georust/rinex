use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParsingError {
    /// SINEX file should start with proper header
    #[error("missing header delimiter")]
    MissingHeader,
    /// Failed to parse Header section
    #[error("invalid header section")]
    InvalidHeader,
    /// Invalid %YYYY:%JJJ:%S epoch format
    #[error("invalid epoch format")]
    EpochFormat,
    /// Closing incorrect section or structure is not correct
    #[error("faulty file structure")]
    FaultySection,
    /// Unknown section / category
    #[error("unknown type of section")]
    UnknownSection(String),
    /// Failed to parse Bias Mode
    #[error("failed to parse bias mode")]
    ParseBiasModeError(#[from] bias::header::BiasModeError),
    /// Failed to parse Determination Method
    #[error("failed to parse determination method")]
    ParseMethodError(#[from] bias::DeterminationMethodError),
    /// Failed to parse time system field
    #[error("failed to parse time system")]
    ParseTimeSystemError(#[from] bias::TimeSystemError),
}
