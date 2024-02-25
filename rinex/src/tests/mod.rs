//! rinex lib test modules
pub mod toolkit;

mod antex;
mod clocks;
mod compression;
#[cfg(feature = "processing")]
mod decimation;
mod decompression;
mod filename;
mod merge;
mod nav;
mod obs;
mod parsing;
mod production;
mod sampling;
mod smoothing;

#[cfg(feature = "meteo")]
mod meteo;

#[cfg(feature = "ionex")]
mod ionex;

#[cfg(feature = "processing")]
mod masking;
