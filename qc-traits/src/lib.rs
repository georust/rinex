//! Traits to generate RINEX and more broadly, GNSS analysis reports.
pub mod html;

pub mod merge;
pub use merge::{Error as MergeError, Merge};
