#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod merge;
pub use merge::{Error as MergeError, Merge};

#[cfg(feature = "processing")]
#[cfg_attr(docsrs, doc(cfg(feature = "processing")))]
mod processing;

#[cfg(feature = "processing")]
pub use processing::{
    Decimate, DecimationError, DecimationFilter, DecimationFilterType, Filter, FilterItem,
    MaskError, MaskFilter, MaskOperand, Masking, Preprocessing, Repair, RepairTrait, Split,
};

#[cfg(feature = "html")]
pub use maud::{html, Markup};

/// HTML reporting
#[cfg(feature = "html")]
#[cfg_attr(docsrs, doc(cfg(feature = "html")))]
pub trait QcHtmlReporting {
    fn render(&self) -> Markup;
}
