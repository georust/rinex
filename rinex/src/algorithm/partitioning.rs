use crate::Duration;

pub trait Partitioning {
    /// Partition dt into subsets of dt duration
    fn partition(&self, dt: Duration) -> Vec<Self> where Self: Sized;
}
