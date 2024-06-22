#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docrs, feature(doc_cfg))]
// pub mod html;

pub mod merge;
pub use merge::{Error as MergeError, Merge};

#[cfg(feature = "processing")]
#[cfg_attr(docrs, doc(cfg(feature = "processing")))]
pub mod processing;
