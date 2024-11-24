//! Split trait
use hifitime::{Duration, Epoch};

/// Implement [Split] to rearrange datasets timewise.
pub trait Split {

    /// [Split]s Self into two at specified [Epoch]
    /// Returns:
    ///  - (a , b) where a <= t and b > t
    fn split(&self, t: Epoch) -> (Self, Self)
    where
        Self: Sized;

    /// [Split]s Self with mutable access.
    /// Modifies Self in place, retaining only <= t.
    /// Returns > t.
    fn split_mut(&mut self, t: Epoch) -> Self;

    /// [Split]s Self into chunks of evenly spaced [Duration]
    fn split_even_dt(&self, dt: Duration) -> Vec<Self>
    where
        Self: Sized;
}
