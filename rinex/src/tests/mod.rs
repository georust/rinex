//! integrated tests
pub mod toolkit;

mod antex;
mod compression;
mod decompression;
mod filename;
mod merge;
mod parsing;
mod production;

#[cfg(feature = "clock")]
mod clock;

#[cfg(feature = "processing")]
mod processing;

#[cfg(feature = "doris")]
mod doris;

#[cfg(feature = "ionex")]
mod ionex;

#[cfg(feature = "meteo")]
mod meteo;

#[cfg(feature = "nav")]
mod nav;

#[cfg(feature = "obs")]
mod obs;
