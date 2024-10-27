//! Split trait
use hifitime::{Duration, Epoch};

/// Implement [Split] to rearrange datasets timewise.
pub trait Split {
    fn split(&self, t: Epoch) -> (Self, Self)
    where
        Self: Sized;

    fn split_mut(&mut self, t: Epoch) -> Self;

    fn split_even_dt(&self, dt: Duration) -> Vec<Self>
    where
        Self: Sized;
}
