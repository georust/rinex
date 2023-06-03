use crate::prelude::*;
use std::collections::HashMap;

pub trait Processing {
    /// Evaluates min() for all Observations and Sv, across all epochs.  
    /// This is only feasible on Observation RINEX.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::Processing;
    /// let let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// let (min_clock, min_observations) = rinex.min();
    /// assert!(min_clock.is_none()); // no clock offset in this dataset
    /// for (svnn, observables) in min_observations {
    ///     for (observable, observation) in observables {
    ///         if observable == Observable::from_str("S1C") {
    ///             assert_eq!(observation, 37.75); // worst SNR was 37.75 dB
    ///         }
    ///     }
    /// }
    /// ```
    fn min(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// Evaluates min() for all Observables across all epochs.
    /// Observation RINEX: min() is performed across vehicules.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::Processing;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// let min = rinex.min_observable();
    /// for (observable, minimum) in min {
    ///     if observable == Observable::Phase("S1C".into()) {
    ///         assert_eq!(minimum, 37.75); // worst overall SNR was 37.75 dB
    ///     }
    /// }
    ///
    /// Meteo RINEX:
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::Processing;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/clar0020.00m")
    ///     .unwrap();
    /// let min = rinex.min();
    /// for (observable, minimum) in min {
    ///     if observable == Observable::Temperature {
    ///         assert_eq!(minimum, 8.4); // lowest temperature observed that day was +4°C
    ///     }
    /// }
    fn min_observable(&self) -> HashMap<Observable, f64>;

    /// Evaluates maximal observation, for all signals and Sv
    fn max(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// Evaluates maximal observation for all signals
    fn max_observable(&self) -> HashMap<Observable, f64>;

    /// Evaluates average value for all signals and Sv
    fn mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// Evalutes average value for all signals (averages accross Sv)
    fn mean_observable(&self) -> HashMap<Observable, f64>;

    /// Evaluates observation skewness per Sv per signal
    fn skewness(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    fn skewness_observable(&self) -> HashMap<Observable, f64>;

    /// Evaluates standard deviation for all signals and Sv
    fn stddev(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// Evaluates standard deviation for all signals
    fn stddev_observable(&self) -> HashMap<Observable, f64>;

    /// Evaluates standard variance for all signals and Sv
    fn stdvar(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// Evaluates standard deviation for all signals
    fn stdvar_observable(&self) -> HashMap<Observable, f64>;

    /// Evaluates nth order central moment for all Signals for all Sv
    fn central_moment(&self, order: u16) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// Evaluates nth order central moment for all Signals, accross Sv
    fn central_moment_observable(&self, order: u16) -> HashMap<Observable, f64>;

    // fn derivative(&self) -> Record;
    // /// computes nth order derivative of this subset
    // fn derivative_nth(&self, order: u8) -> A;
}
