//! RINEX compression / decompression module
use crate::sv;
use crate::header;
use crate::is_comment;
use crate::types::Type;
use thiserror::Error;
use std::str::FromStr;
use std::collections::HashMap;

mod numdiff;
mod textdiff;

pub mod compressor;
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
    #[error("Failed to rework epoch to match standards")]
    EpochReworkFailure,
    #[error("Malformed epoch description (#nb sv)")]
    MalformedEpochDescriptor,
    #[error("Vehicule identification failed")]
    VehiculeIdentificationError,
    #[error("Malformed f64 data")]
    MalformedObservable,
    #[error("Malformed epoch content (#nb of observables)")]
    MalformedEpochBody,
    #[error("numdiff error")]
    NumDiffError(#[from] numdiff::Error),
    #[error("failed to identify sat. vehicule")]
    SvError(#[from] sv::Error),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
}
