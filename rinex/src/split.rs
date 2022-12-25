use thiserror::Error;
use crate::{Epoch, Duration};

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("this record type is not indexed by epoch")]
    NoEpochIteration,
}

pub trait Split<T> {
    /// Splits Self at desired epoch,
    /// retaining |e(k) < epoch| ("before"), as left component,
    /// and |e(k) >= epoch| ("inclusive after"), as right component.
    /// Fails if self is not indexed by `Epoch`.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::split::Split;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
    ///     .unwrap();
    /// let epoch = Epoch::from_gregorian_utc(2021, 01, 01, 0, 1, 00, 00);
    /// let (rnx_a, rnx_b) = rnx.split(epoch)
    ///     .unwrap();
    /// assert_eq!(rnx_a.epochs().len(), 2);
    /// assert_eq!(rnx_b.epochs().len(), rnx.epochs().len() - 2);
    /// ```
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), Error>
    where
        Self: Sized;
	
	/// Splits Self into a serie of epoch of equal durations
	fn split_dt(&self, dt: Duration) -> Result<Vec<Self>, Error>
	where
		Self: Sized;
}
