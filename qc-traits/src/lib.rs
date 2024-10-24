#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod merge;
pub use merge::{Error as MergeError, Merge};

mod split;
pub use split::Split;

#[cfg(feature = "processing")]
#[cfg_attr(docsrs, doc(cfg(feature = "processing")))]
mod processing;

#[cfg(feature = "processing")]
pub use processing::{
    Decimate, DecimationError, DecimationFilter, DecimationFilterType, Filter, FilterItem,
    MaskError, MaskFilter, MaskOperand, Masking, Preprocessing, Repair, RepairTrait,
};
