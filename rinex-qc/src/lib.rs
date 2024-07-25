//! RINEX/GNSS data analysis library
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![cfg_attr(docrs, feature(doc_cfg))]

extern crate gnss_rs as gnss;
extern crate rinex_qc_traits as qc_traits;

mod cfg;

#[cfg(feature = "plot")]
pub mod plot;

mod context;
mod report;

pub mod prelude {
    #[cfg(feature = "plot")]
    pub use crate::plot::{Marker, MarkerSymbol, Mode, Plot};
    pub use crate::{
        cfg::QcConfig,
        context::{ProductType, QcContext},
        report::{QcExtraPage, QcReport},
    };
    pub use maud::{html, Markup, Render};
    pub use qc_traits::processing::{Filter, Preprocessing};
}
