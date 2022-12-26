use crate::Duration;

pub trait Partitioning {
    /// Partition self into subsets of Duration `dt`
    fn partition_dt(&self, dt: Duration) -> Vec<Self> where Self: Sized;
}
