//! RINEX Quality analysis library
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![cfg_attr(docrs, feature(doc_cfg))]

extern crate gnss_rs as gnss;
extern crate rinex_qc_traits as qc_traits;

mod cfg;
pub use cfg::QcConfig;

use qc_traits::html::*;

mod context;
pub use context::{ProductType, QcContext};

mod report;
pub use qc_traits::html::RenderHtml;
pub use report::QcReport; // re-export

pub mod prelude {
    pub use crate::context::{ProductType, QcContext};
    pub use qc_traits::{
        html::RenderHtml,
        processing::{Filter, Preprocessing},
    };
}
