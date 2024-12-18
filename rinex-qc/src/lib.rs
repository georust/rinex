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

#[derive(Debug, Error)]
pub enum QcError {
    #[error("i/o error")]
    IO,
    #[error("almanac error")]
    Alamanac(#[from] AlmanacError),
    #[error("alamanc setup issue: {0}")]
    MetaAlamanac(#[from] MetaAlmanacError),
    #[error("frame model error: {0}")]
    FrameModel(#[from] PlanetaryDataError),
    #[error("internal indexing/sorting issue")]
    DataIndexingIssue,
    #[error("failed to extend gnss context")]
    ContextExtensionError(#[from] MergeError),
    #[error("non supported file format")]
    NonSupportedFormat,
    #[error("failed to determine file name")]
    FileName,
    #[error("failed to determine file extension")]
    FileExtension,
    #[error("invalid rinex format")]
    RinexParsingError(#[from] RinexParsingError),
}
