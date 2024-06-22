//! RINEX Quality analysis library
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![cfg_attr(docrs, feature(doc_cfg))]

extern crate gnss_rs as gnss;
extern crate rinex_qc_traits as qc_traits;

mod cfg;

#[cfg(feature = "plot")]
mod plot;

mod context;
mod report;

pub mod prelude {
    pub use crate::cfg::QcConfig;
    pub use crate::context::{ProductType, QcContext};
    pub use crate::report::QcReport;
    pub use maud::Render;
    pub use qc_traits::processing::{Filter, Preprocessing};
}
