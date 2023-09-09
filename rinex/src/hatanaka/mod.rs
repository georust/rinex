//! RINEX compression / decompression module
use crate::sv;
use thiserror::Error;

pub mod compressor;
pub mod numdiff;
pub mod textdiff;
pub use compressor::Compressor;

pub mod decompressor;
pub use decompressor::Decompressor;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("This is not a CRX file")]
    NotACrinex,
    #[error("This is not an Observation file")]
    NotObsRinexData,
    #[error("Non supported CRX revision")]
    NonSupportedCrxVersion,
    #[error("First epoch not delimited by \"&\"")]
    FaultyCrx1FirstEpoch,
    #[error("First epoch not delimited by \">\"")]
    FaultyCrx3FirstEpoch,
    #[error("Failed to parse clock offset init order")]
    ClockOffsetOrderError,
    #[error("Failed to parse clock offset value")]
    ClockOffsetValueError,
    #[error("Recovered epoch content seems faulty")]
    FaultyRecoveredEpoch,
    #[error("failed to reconstruct epoch description")]
    EpochConstruct,
    #[error("Malformed epoch description (#nb sv)")]
    MalformedEpochDescriptor,
    #[error("Vehicle identification failed")]
    VehicleIdentificationError,
    #[error("Malformed epoch content (#nb of observables)")]
    MalformedEpochBody,
    #[error("numdiff error")]
    NumDiffError(#[from] numdiff::Error),
    #[error("sv parsing error")]
    SvParsing(#[from] sv::ParsingError),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
}
