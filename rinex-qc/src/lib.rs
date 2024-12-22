#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate log;

extern crate gnss_rs as gnss;
extern crate rinex_qc_traits as qc_traits;

mod cfg;

pub mod plot;

mod context;
mod navigation;
mod report;

pub mod prelude {
    pub use crate::{
        cfg::{ConfigError, QcConfig, QcReportType},
        context::QcContext,
        report::{QcExtraPage, QcReport},
        QcError,
    };
    // Pub re-export
    pub use crate::plot::{Marker, MarkerSymbol, Mode, Plot};
    pub use maud::{html, Markup, Render};
    pub use qc_traits::{Filter, Merge, MergeError, Preprocessing, Repair, RepairTrait, Split};
    pub use rinex::prelude::{nav::Almanac, Error as RinexError, Rinex};
    #[cfg(feature = "sp3")]
    pub use sp3::prelude::{Error as SP3Error, SP3};
    pub use std::path::Path;
}

use qc_traits::MergeError;
use thiserror::Error;

use anise::{
    almanac::{metaload::MetaAlmanacError, planetary::PlanetaryDataError},
    errors::AlmanacError,
};

use rinex::prelude::ParsingError as RinexParsingError;

/// [QcCtxError] wraps I/O and context creation errors,
/// basically early deployment errors and file related issues.
#[derive(Debug, Error)]
pub enum QcCtxError {
    #[error("i/o error")]
    IO,
    #[error("almanac error: {0}")]
    Alamanac(#[from] AlmanacError),
    #[error("alamanc setup issue: {0}")]
    MetaAlamanac(#[from] MetaAlmanacError),
    #[error("frame model error: {0}")]
    FrameModel(#[from] PlanetaryDataError),
    #[error("data sorting issue")]
    DataIndexing,
    #[error("data stacking")]
    Stacking(#[from] MergeError),
    #[error("non supported format")]
    NonSupportedFormat,
    #[error("file name determination")]
    FileName,
    #[error("file extension determination")]
    FileExtension,
    #[error("rinex parsing")]
    RinexParsing(#[from] RinexParsingError),
}

/// [QcError] wraps all post processing errors,
/// so actual data exploitation.
#[derive(Debug, Error)]
pub enum QcError {
    #[error("no ephemeris source")]
    NoEphemeris,
    #[error("no signal source")]
    NoSignal,
    #[error("signal source initlization")]
    SignalSourceInit,
}