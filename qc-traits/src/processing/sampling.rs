//! Sampling related features
use hifitime::{Duration, Epoch};

use crate::processing::{Histogram, HistogramEntry};

/// Sampling Analysis Trait
pub trait Sampling {
    /// Returns first [Epoch]: describes the first data point in Time.
    fn first_epoch(&self) -> Option<Epoch>;
    /// Returns last [Epoch]: describes the last data point in Time.
    fn last_epoch(&self) -> Option<Epoch>;
    /// Returns true if steady sampling occured (ie., not [DataGap]s were found)
    fn steady_sampling(&self) -> bool {
        self.data_gaps().count() == 0
    }
    /// Returns (theoretical) sampling period, expressed as [Duration].
    /// We distinguish theoretical sampling period from actual sampling period.
    /// The actual sampling period may not be completely steady due to context issues.
    fn sampling_period(&self) -> Option<Duration>;
    /// Returns (theoretical) sampling rate, expressed in Hertz.
    /// We distinguish theoretical sampling rate from actual sampling rate.
    /// The actual sampling rate may not be completely steady due to context issues.
    fn sampling_rate_hz(&self) -> Option<f64> {
        let period_s = self.sampling_period()?.to_seconds();
        Some(1.0 / period_s)
    }
    /// [Histogram] analysis to study how the sampling period varies
    /// accross a measurement (reflects context issues).
    fn sampling_period_histogram(&self) -> Histogram<Duration>;
    /// [Histogram] analysis to study how the sampling rate varies
    /// accross a measurement (reflects context issues).
    fn sampling_rate_histogram(&self) -> Histogram<f64> {
        self.sampling_period_histogram()
            .iter()
            .map(|h| HistogramEntry {
                value: 1.0 / h.value.to_seconds(),
                population: h.population,
            })
            .collect()
    }
    /// [DataGap]s analysis: returns all [DataGap] that were determined.
    fn data_gaps(&self) -> Box<dyn Iterator<Item = DataGap> + '_>;
    /// Returns [Duration] of longest [DataGap].
    /// Returns [Duration::ZERO] if no [DataGap]s were found.
    fn longest_gap(&self) -> Duration {
        let gaps = self.data_gaps().collect::<Vec<_>>();
        if gaps.len() == 0 {
            Duration::ZERO
        } else {
            let longest = gaps.iter().max_by(|a, b| b.duration.cmp(&a.duration));
            longest.unwrap().duration
        }
    }
    /// Returns [Duration] of shortest [DataGap].
    /// Returns [Duration::ZERO] if no [DataGap]s were found.
    fn shortest_gap(&self) -> Duration {
        let gaps = self.data_gaps().collect::<Vec<_>>();

        if gaps.len() == 0 {
            Duration::ZERO
        } else {
            let longest = gaps.iter().max_by(|a, b| a.duration.cmp(&b.duration));
            longest.unwrap().duration
        }
    }
}

/// DataGap describes a discontinuity in Time
pub struct DataGap {
    /// Start of this gap. When describing [DataGap]s,
    /// the starting point is the "first" Instant where data is missing
    /// (= excluded data point).
    pub start: Epoch,
    /// Duration of this gap
    pub duration: Duration,
}

impl DataGap {
    /// Returns End of this [DataGap].
    /// When describing [DataGap]s, the [DataGap] end point
    /// is the last Instant where data is missing. The next point
    /// will have data.
    pub fn gap_end(&self) -> Epoch {
        self.start + self.duration
    }
}
