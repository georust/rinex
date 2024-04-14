use gnss_rtk::prelude::{Duration, Epoch};

mod orbit;
pub use orbit::Interpolator as OrbitInterpolator;

mod time;
pub use time::Interpolator as TimeInterpolator;

/// Shared Interpolator buffer behavior
pub trait Buffer<T> {
    /// Memory allocation.
    fn malloc(order: usize) -> Self;
    /// Internal buffer is at capacity: ready to work.
    fn at_capacity(&self) -> bool;
    /// Internal buffer depth
    fn len(&self) -> usize;
    /// Grab symbol per index
    fn get(&self, idx: usize) -> Option<&(Epoch, T)>;
    /// Grab latest symbol in time
    fn latest(&self) -> Option<&(Epoch, T)>;
    /// Clear internal Buffer
    fn clear(&mut self);
    /// Push one symbol into internal buffer
    fn push(&mut self, x_j: (Epoch, T));
    /// Latest dt in buffer
    fn dt(&self) -> Option<(Epoch, Duration)> {
        if self.len() > 1 {
            let (z2, _) = self.get(self.len() - 2)?;
            let (z1, _) = self.get(self.len() - 1)?;
            Some((*z1, *z1 - *z2))
        } else {
            None
        }
    }
    /// Fill internal Buffer. Does not tolerate data gaps.
    /// Panic on chronological order mixup.
    fn fill(&mut self, x_j: (Epoch, T)) {
        if let Some((last, dt)) = self.dt() {
            if (x_j.0 - last).to_seconds().is_sign_positive() {
                if (x_j.0 - last) > dt {
                    warn!("{} - {} gap detected - buffer reset", x_j.0, x_j.0 - last);
                    self.clear();
                    self.push(x_j);
                } else {
                    self.push(x_j);
                }
            } else {
                panic!("samples should be pushed in chronological order");
            }
        } else {
            self.push(x_j);
        }
    }
    /// Internal buffer snapshot
    fn snapshot(&self) -> &[(Epoch, T)];
}
