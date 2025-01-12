#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate log;

extern crate gnss_rs as gnss;
extern crate rinex_qc_traits as qc_traits;

pub mod cfg;

mod analysis;
mod context;

#[cfg(feature = "nav")]
#[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
mod navigation;

#[cfg(feature = "html")]
#[cfg_attr(docsrs, doc(cfg(feature = "html")))]
pub mod html;

pub mod prelude {
    pub use crate::{analysis::QcAnalysis, cfg::QcConfig, context::QcContext, QcError};

    pub use qc_traits::{Filter, Merge, MergeError, Preprocessing, Repair, RepairTrait, Split};

    pub use rinex::prelude::{Error as RinexError, Rinex};

    #[cfg(feature = "nav")]
    pub use gnss_rtk::prelude::{Config as RTKConfig, Method as RTKMethod, PVTSolutionType};

    #[cfg(feature = "nav")]
    pub use cggtts::prelude::Track as CggttsTrack;

    #[cfg(feature = "nav")]
    pub use rinex::prelude::nav::{Almanac, Orbit};

    #[cfg(feature = "html")]
    pub use maud::{html, Markup};

    #[cfg(feature = "html")]
    pub use crate::html::plot::{Marker, MarkerSymbol, Mode, Plot};

    #[cfg(feature = "html")]
    pub use qc_traits::QcHtmlReporting;

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
    #[error("ephemeris source design")]
    EphemerisSource,
    #[error("failed to determine rx position")]
    RxPosition,
    #[error("orbital source design")]
    OrbitalSource,
    #[error("clock source design")]
    ClockSource,
    #[error("no signal source")]
    SignalSource,
}

#[cfg(feature = "nav")]
use gnss_rtk::prelude::Error as RTKError;

/// [RTKCggttsError] is returned by [NavCggttsSolver]
/// and basically combines [RTKError] and CGGTTS tracking errors
#[derive(Debug, Error)]
pub enum QcRtkCggttsError {
    #[error("rtk error: {0}")]
    RTK(#[from] RTKError),
    #[error("dummy")]
    Dumy,
}
