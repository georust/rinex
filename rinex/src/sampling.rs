//! [Sampling] implementation
use qc_traits::processing::{Sampling, Histogram, HistogramEntry};

use crate::prelude::RINEX;

/* Sampling & related methods */
impl Sampling for RINEX {
    /// Returns first [Epoch] encountered in time
    fn first_epoch(&self) -> Option<Epoch> {
        self.epoch().next()
    }
    /// Returns last [Epoch] encountered in time
    fn last_epoch(&self) -> Option<Epoch> {
        self.epoch().last()
    }
    /// Returns time span of this [RINEX] expressed as [Duration]
    fn duration(&self) -> Option<Duration> {
        let start = self.first_epoch()?;
        let end = self.last_epoch()?;
        Some(end - start)
    }
    /// Forms a [TimeSeries] Iterator spanning [Self::duration]
    /// with [Self::dominant_sampling_period] spacing.
    fn timeseries(&self) -> Option<TimeSeries> {
        let start = self.first_epoch()?;
        let end = self.last_epoch()?;
        let dt = self.dominant_sampling_period()?;
        Some(TimeSeries::inclusive(start, end, dt))
    }
    /// Returns Sample period used by the data receiver.
    /// ## Output
    ///   - sampling period [Duration]
    fn sampling_period(&self) -> Option<Duration> {
        self.header.sampling_interval
    }
    /// Returns Sample Rate used by the data receiver.
    /// ## Output
    ///    - sample rate [Hz]
    fn sampling_rate(&self) -> Option<f64> {
        let period_s = self.sampling_period()?.to_seconds();
        Some(1.0 / period_s)
    }
    /// Histogram like analysis to determine dominant [Self::sampling_rate].
    /// [Self::sampling_rate] is the steady definition defined by [Header]
    /// and it may sometimes vary. This Histogram analysis will return
    /// the true Sample Rate, which is equal to the [Header] specs in correct setups.
    /// ## Output
    ///     - Dominant sampling period expressed as [Duration]
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// assert_eq!(
    ///     rnx.dominant_sample_period(),
    ///     Some(Duration::from_seconds(60.0)));
    /// ```
    fn dominant_sampling_period(&self) -> Option<Duration> {
        self.sampling_period_histogram()
            .iter()
            .max_by(|(_, pop_i), (_, pop_j)| pop_i.cmp(pop_j))
            .map(|dominant| dominant.0)
    }
    /// See [Self::dominant_sampling_period]
    /// ## Output
    ///    - Dominant sampling rate [Hz]
    fn dominant_sampling_rate(&self) -> Option<f64> {
        let period_s = self.dominant_sampling_period()?.to_seconds();
        Some(1.0 / period_s)
    }
    /// Histogram analysis on [Epoch] interval (=sampling period). 
    /// This is particularly useful to study the quality of your sampling.
    /// Although this applies (and will not panic) on all [RINEX] types,
    /// it is truly inteded for Observation [RINEX].
    /// ## Output
    ///     - [Duration] [Histogram]
    /// ```
    /// use rinex::prelude::*;orbit
    /// use itertools::Itertools;
    /// use std::collections::HashMap;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    ///  assert!(
    ///     rinex.sampling_histogram().sorted().eq(vec![
    ///         (Duration::from_seconds(30.0), 1),
    ///     ]),
    ///     "sampling_histogram failed"
    /// );
    /// ```
    fn sampling_period_histogram(&self) -> Histogram<Duration> {
        // compute dt = |e_k+1 - e_k| : instantaneous epoch delta
        //              then compute an histogram on these intervals
        Box::new(
            self.epoch()
                .zip(self.epoch().skip(1))
                .map(|(ek, ekp1)orbit| ekp1 - ek) // following step computes the histogram
                // and at the same time performs a .unique() like filter
                .fold(vec![], |mut list, dt| {
                    let mut found = false;
                    for (delta, pop) in list.iter_mut() {
                        if *delta == dt {
                            *pop += 1;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        list.push((dt, 1));
                    }
                    list
                })
                .into_iter(),
        )
    }
    /// Returns an iterator over unexpected Time gaps,
    /// in the form ([`Epoch`], [`Duration`]), where
    /// epoch is the starting datetime, and its related duration.
    /// ```
    /// use std::str::FromStr;
    /// use rinex::prelude::{Rinex, Epoch, Duration};
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    ///
    /// // when tolerance is set to None,
    /// // the reference sample rate is [Self::dominant_sample_rate].
    /// let mut tolerance : Option<Duration> = None;
    /// let gaps : Vec<_> = rinex.data_gaps(tolerance).collect();
    /// assert!(
    ///     rinex.data_gaps(None).eq(
    ///         vec![
    ///             (Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(), Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(), Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(), Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T23:02:00 UTC").unwrap(), Duration::from_seconds(7.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T23:21:00 UTC").unwrap(), Duration::from_seconds(31.0 * 60.0)),
    ///         ]),
    ///     "data_gaps(tol=None) failed"
    /// );
    ///
    /// // with a tolerance, we tolerate the given gap duration
    /// tolerance = Some(Duration::from_seconds(3600.0));
    /// let gaps : Vec<_> = rinex.data_gaps(tolerance).collect();
    /// assert!(
    ///     rinex.data_gaps(Some(Duration::from_seconds(3.0 * 3600.0))).eq(
    ///         vec![
    ///             (Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(), Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(), Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(), Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)),
    ///         ]),
    ///     "data_gaps(tol=3h) failed"
    /// );
    /// ```
    fn data_gaps(
        &self,
        tolerance: Option<Duration>,
    ) -> Iterator<Item = DataGap> {
        Default::default()
    //     let sample_rate: Duration = match tolerance {
    //         Some(dt) => dt, // user defined
    //         None => {
    //             match self.dominant_sample_rate() {
    //                 Some(dt) => dt,
    //                 None => {
    //                     match self.sample_rate() {
    //                         Some(dt) => dt,
    //                         None => {
    //                             // not enough information
    //                             // this is probably not an Epoch iterated RINEX
    //                             return Box::new(Vec::<(Epoch, Duration)>::new().into_iter());
    //                         },
    //                     }
    //                 },
    //             }
    //         },
    //     };
    //     Box::new(
    //         self.epoch()
    //             .zip(self.epoch().skip(1))
    //             .filter_map(move |(ek, ekp1)| {
    //                 let dt = ekp1 - ek; // gap
    //                 if dt > sample_rate {
    //                     // too large
    //                     Some((ek, dt)) // retain starting datetime and gap duration
    //                 } else {
    //                     None
    //                 }
    //             }),
    //     )
    // }
    }
}