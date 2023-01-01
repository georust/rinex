use crate::Duration;

pub trait Decimation<T> {
    fn decim_by_ratio_mut(&mut self, r: u32);

    /// [Decimation::decim_by_ratio_mut] immutable implementation.
    fn decim_by_ratio(&self, r: u32) -> Self;

    fn decim_by_interval_mut(&mut self, interval: Duration);

    /// [Decimation::decim_by_interval_mut] immutable implementation.
    fn decim_by_interval(&self, interval: Duration) -> Self;

    /// Decimates Self so sample rate matches `rhs`
    fn decim_match_mut(&mut self, rhs: &Self);
    /// Copies and decimates Self so sample rate matches `rhs`
    fn decim_match(&self, rhs: &Self) -> Self;
}
