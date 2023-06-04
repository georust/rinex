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
    fn statistical_ops(
        &self,
        ops: StatisticalOps,
    ) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// If you're interested in the .min() of this RINEX
    /// dataset for example, invoke and refer to .min_observable() directly.
    /// This is only feasible on either Observation or Meteo RINEX.
    fn statistical_observable_ops(&self, ops: StatisticalOps) -> HashMap<Observable, f64>;
    /// Evalutes min() across all Epochs, for each individual physics.   
    /// This is only feasible on Observation RINEX.
    /// ```
    /// use rinex::*;
    /// use rinex::prelude::*;
    /// use std::str::FromStr; // observable!
    /// use rinex::processing::*; // .min()
    ///
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// let (min_clock, min_observations) = rinex.min();
    /// assert!(min_clock.is_none()); // no clock offset in this dataset
    /// for (svnn, observables) in min_observations {
    ///     for (observable, min_observation) in observables {
    ///         if observable == observable!("S1C") {
    ///         }
    ///     }
    /// }
    /// ```
    fn min(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// Evaluates min() for all Observables across all epochs.
    /// This is only feasible on either Observation or Meteo RINEX.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::processing::*; // .min_observable()
    ///
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// let min = rinex.min_observable();
    /// for (observable, minimum) in min {
    ///     if observable == Observable::Phase("S1C".into()) {
    ///         assert_eq!(minimum, 44.5); // worst overall SNR was 44.5 dB
    ///     }
    /// }
    /// ```
    ///
    /// Meteo RINEX:
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::processing::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/clar0020.00m")
    ///     .unwrap();
    /// let min = rinex.min_observable();
    /// for (observable, minimum) in min {
    ///     if observable == Observable::Temperature {
    ///         assert_eq!(minimum, 8.4); // lowest temperature observed that day was +8.4°C
    ///     }
    /// }
    fn min_observable(&self) -> HashMap<Observable, f64>;

    /// Evaluates |.min()| on Self.
    /// This is only feasible on Observation RINEX.  
    /// See [Processing::min] for more detail.
    fn abs_min(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    /// Evalutes |.min()| across Sv and Epochs.  
    /// This is only feasible on Observation RINEX.  
    /// See [Processing::min_observable] for more detail.
    fn abs_min_observable(&self) -> HashMap<Observable, f64>;

    /// Evalutes max() across all Epochs, for each individual physics.
    /// This is only feasible on Observation RINEX.  
    /// See [Processing::min_observable] for more detail.
    fn max(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    /// Evalutes |.max()| across Sv and Epochs.  
    /// This is only feasible on Observation RINEX.  
    /// See [Processing::min] for more detail.
    fn abs_max(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    /// Evalutes .max() across Sv and Epochs.  
    /// This is only feasible on either Observation or Meteo RINEX.
    /// See [Processing::min_observable] to understand how results are sorted.
    fn max_observable(&self) -> HashMap<Observable, f64>;

    /// Evalutes |.max()| across Sv and Epochs.  
    /// This is only feasible on either Observation or Meteo RINEX.
    /// See [Processing::min_observable] to understand how results are sorted.
    fn abs_max_observable(&self) -> HashMap<Observable, f64>;

    /// Computes mean for both clock offsets (if feasible)
    /// and across all Epochs for each individual physics.
    /// This is only feasible on either Observation RINEX.
    /// See [Processing::min] to understand how results are sorted.
    fn mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    /// Computes quadratic mean for both clock offsets (if feasible)
    /// and across all Epochs for each individual physics.
    /// This is only feasible on either Observation RINEX.
    /// See [Processing::min] to understand how results are sorted.
    fn quadratic_mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    /// Computes harmonic mean for both clock offsets (if feasible)
    /// and across all Epochs for each individual physics.
    /// This is only feasible on either Observation RINEX.
    /// See [Processing::min] to understand how results are sorted.
    fn harmonic_mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    /// Computes geometric mean for both clock offsets (if feasible)
    /// and across all Epochs for each individual physics.
    /// This is only feasible on either Observation RINEX.
    /// See [Processing::min] to understand how results are sorted.
    fn geometric_mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    /// Computes mean for each individual physics.
    /// For OBS RINEX, measurements are averaged across vehicles.
    /// This is only feasible on either Observation or Meteo RINEX.  
    /// See [Processing::min_observable] to understand how results are sorted.
    fn mean_observable(&self) -> HashMap<Observable, f64>;

    /// Computes quadratic mean for each individual physics.
    /// For OBS RINEX, measurements are averaged across vehicles.
    /// This is only feasible on either Observation or Meteo RINEX.  
    /// See [Processing::min_observable] to understand how results are sorted.
    fn quadratic_mean_observable(&self) -> HashMap<Observable, f64>;

    /// Computes harmonic mean for each individual physics.
    /// For OBS RINEX, measurements are averaged across vehicles.
    /// This is only feasible on either Observation or Meteo RINEX.  
    /// See [Processing::min_observable] to understand how results are sorted.
    fn harmonic_mean_observable(&self) -> HashMap<Observable, f64>;

    /// Computes geometric mean for each individual physics.
    /// For OBS RINEX, measurements are averaged across vehicles.
    /// This is only feasible on either Observation or Meteo RINEX.  
    /// See [Processing::min_observable] to understand how results are sorted.
    fn geometric_mean_observable(&self) -> HashMap<Observable, f64>;

    // fn skewness(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    // fn skewness_observable(&self) -> HashMap<Observable, f64>;

    /// Computes standard deviation for both clock offsets (if feasible),
    /// and each individual physics per Epoch and per Sv.
    /// This is only feasible on OBS RINEX.  
    /// See [Processing::min] to understand how results are sorted.
    fn std_dev(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    /// Computes standard variance for both clock offsets (if feasible),
    /// and each individual physics per Epoch and per Sv.
    /// This is only feasible on OBS RINEX.  
    /// See [Processing::min] to understand how results are sorted.
    fn variance(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);

    //fn central_moment(&self, order: u16) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    //fn central_moment_observable(&self, order: u16) -> HashMap<Observable, f64>;
    // fn derivative(&self) -> Record;
    // /// computes nth order derivative of this subset
    // fn derivative_nth(&self, order: u8) -> A;
}
