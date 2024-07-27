#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docrs, feature(doc_cfg))]

extern crate gnss_rs as gnss;
extern crate rinex_qc_traits as qc_traits;

mod cfg;

#[cfg(feature = "plot")]
pub mod plot;

mod context;
mod report;

pub mod prelude {
    pub use crate::{
        cfg::{QcConfig, QcReportType},
        context::{ProductType, QcContext},
        report::{QcExtraPage, QcReport},
    };
    // Pub re-export
    #[cfg(feature = "plot")]
    pub use crate::plot::{Marker, MarkerSymbol, Mode, Plot};
    pub use maud::{html, Markup, Render};
    pub use qc_traits::processing::{Filter, Preprocessing};
    pub use rinex::prelude::{Error as RinexError, Rinex};
    #[cfg(feature = "sp3")]
    pub use sp3::prelude::{Error as SP3Error, SP3};
    pub use std::path::Path;
}
