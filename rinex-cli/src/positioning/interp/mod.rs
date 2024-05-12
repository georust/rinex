mod orbit;
pub use orbit::Interpolator as OrbitInterpolator;

mod time;
pub use time::Interpolator as TimeInterpolator;

use rinex::prelude::{Duration, Epoch};

/// Interpolators internal buffer.
/// Both interpolators, whether it be Temporal or Position, both work on 3D values represented as f64 double precision.
pub trait Buffer {
    /// Memory allocation
    fn malloc(size: usize) -> Self;
    /// Return current number of symbols
    fn len(&self) -> usize;
    /// Return symbol by index
    fn get(&self, index: usize) -> Option<&(Epoch, (f64, f64, f64))>;
    /// Clear all symbols
    fn clear(&mut self);
    /// New new symbol
    fn push(&mut self, x_j: (Epoch, (f64, f64, f64)));
    /// Returns internal symbols
    fn snapshot(&self) -> &[(Epoch, (f64, f64, f64))];
    /// Returns true if an interpolation of this order is feasible @ t
    fn feasible(&self, order: usize, t: Epoch) -> bool;
    /// Returns direct output in rare cases where Interpolation is not needed.
    /// This avoids introduction extra bias in the measurement, due to the interpolation process.
    fn direct_output(&self, t: Epoch) -> Option<&(f64, f64, f64)> {
        self.snapshot()
            .iter()
            .filter_map(|(k, v)| if *k == t { Some(v) } else { None })
            .reduce(|k, _| k)
    }
    /// Returns mutable internal symbols
    fn snapshot_mut(&mut self) -> &mut [(Epoch, (f64, f64, f64))];
    /// Returns latest interval
    fn last_dt(&self) -> Option<(Epoch, Duration)> {
        if self.len() > 1 {
            let (z2, _) = self.get(self.len() - 2)?;
            let (z1, _) = self.get(self.len() - 1)?;
            Some((*z1, *z1 - *z2))
        } else {
            None
        }
    }
    /// Streams data in, in chronological order with gap intolerance.
    fn fill(&mut self, x_j: (Epoch, (f64, f64, f64))) {
        if let Some((last, dt)) = self.last_dt() {
            if (x_j.0 - last).to_seconds().is_sign_positive() {
                // NB Should we make gap tolerance more flexible ?
                if (x_j.0 - last) > dt {
                    warn!("{} - {} gap detected - buffer reset", x_j.0, x_j.0 - last);
                    self.clear();
                    self.push(x_j);
                } else {
                    self.push(x_j);
                }
            } else {
                panic!("symbols should be streamed in chronological order");
            }
        } else {
            self.push(x_j);
        }
    }
}
