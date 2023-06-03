use crate::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub enum StatisticalOps {
    Max,
    Min,
    MaxAbs,
    MinAbs,
    Variance,
    StdDev,
    Mean,
    QuadMean,
    HarmMean,
    GeoMean,
}

pub trait Processing {
    /// If you're interested in the .min() of this RINEX
    /// dataset for example, invoke and refer to .min() directly.  
    /// <!> Only implemented on Observation RINEX <!>
    fn statistical_ops(&self, ops: StatisticalOps) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// If you're interested in the .min() of this RINEX
    /// dataset for example, invoke and refer to .min_observable() directly.
    /// This is only feasible on either Observation or Meteo RINEX.
    fn statistical_observable_ops(&self, ops: StatisticalOps) -> HashMap<Observable, f64>;
    /// Evaluates min() for all Observations and Sv, and also for Clock Offsets, across all epochs.  
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
    /// This is only feasible on either Observation or Meteo RINEX.
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
    /// let min = rinex.min_observable();
    /// for (observable, minimum) in min {
    ///     if observable == Observable::Temperature {
    ///         assert_eq!(minimum, 8.4); // lowest temperature observed that day was +8.4°C
    ///     }
    /// }
    fn min_observable(&self) -> HashMap<Observable, f64>;

    /// This is only feasible on Observation RINEX.
    fn abs_min(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// This is only feasible on either Observation or Meteo RINEX.
    fn abs_min_observable(&self) -> HashMap<Observable, f64>;

    /// This is only feasible on Observation RINEX.
    fn max(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// This is only feasible on Observation RINEX.
    fn abs_max(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    
    /// Evaluates max() for all Observables across all epochs.  
    /// This is only feasible on either Observation or Meteo RINEX.
    ///
    /// Meteo RINEX:
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::Processing;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/clar0020.00m")
    ///     .unwrap();
    /// let max = rinex.max_observable();
    /// for (observable, max) in max {
    ///     if observable == Observable::Temperature {
    ///         assert_eq!(minimum, 16.2); // highest temperature observed that day was +16.2°C
    ///     }
    /// }
    fn max_observable(&self) -> HashMap<Observable, f64>;
    /// This is only feasible on either Observation or Meteo RINEX.
    fn abs_max_observable(&self) -> HashMap<Observable, f64>;

    /// This is only feasible on Observation RINEX.
    fn mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// This is only feasible on Observation RINEX.
    fn quadratic_mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// This is only feasible on Observation RINEX.
    fn harmonic_mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// This is only feasible on Observation RINEX.
    fn geometric_mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    /// This is only feasible on either Observation or Meteo RINEX.
    fn mean_observable(&self) -> HashMap<Observable, f64>;
    /// This is only feasible on either Observation or Meteo RINEX.
    fn quadratic_mean_observable(&self) -> HashMap<Observable, f64>;
    /// This is only feasible on either Observation or Meteo RINEX.
    fn harmonic_mean_observable(&self) -> HashMap<Observable, f64>;
    /// This is only feasible on either Observation or Meteo RINEX.
    fn geometric_mean_observable(&self) -> HashMap<Observable, f64>;

    // fn skewness(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    // fn skewness_observable(&self) -> HashMap<Observable, f64>;

    /// This is only feasible on Observation RINEX.
    fn std_dev(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// This is only feasible on Observation RINEX.
    fn variance(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    //fn central_moment(&self, order: u16) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    //fn central_moment_observable(&self, order: u16) -> HashMap<Observable, f64>;
    // fn derivative(&self) -> Record;
    // /// computes nth order derivative of this subset
    // fn derivative_nth(&self, order: u8) -> A;
}
