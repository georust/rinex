//! RINEX File splitting operation
use crate::{Duration, Epoch};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("this record type is not indexed by epoch")]
    NoEpochIteration,
    #[error("this record does not contained specified epoch")]
    NonExistingEpoch,
}

pub trait Split {
    /// Splits Self at desired epoch,
    /// retaining |e(k) < epoch| ("before"), as left component,
    /// and |e(k) >= epoch| ("inclusive after"), as right component.
    /// Fails if self is not indexed by `Epoch`.
    /// ```
    /// use rinex::Split; // .split()
    /// use rinex::prelude::*; // Rinex
    /// use std::str::FromStr;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
    ///     .unwrap();
    /// let epoch = Epoch::from_str("2021-01-01T00:01:00 GPST")
    ///   .unwrap();
    /// let (rnx_a, rnx_b) = rnx.split(epoch)
    ///     .unwrap();
    /// let epochs   : Vec<_> = rnx.epoch().collect();
    /// let a_epochs : Vec<_> = rnx_a.epoch().collect();
    /// let b_epochs : Vec<_> = rnx_b.epoch().collect();
    /// assert_eq!(a_epochs.len(), 2);
    /// assert_eq!(b_epochs.len(),  epochs.len() -2);
    /// ```
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), Error>
    where
        Self: Sized;

    /// Splits Self into a serie of epoch of equal durations
    fn split_dt(&self, dt: Duration) -> Result<Vec<Self>, Error>
    where
        Self: Sized;
}
