//! Observation specific high level methods
use crate::prelude::{ClockObservation, ObsKey, Observation, Rinex, SignalObservation};

use std::collections::btree_map::{Iter, IterMut, Keys};

impl Rinex {
    /// Returns [ObsKey] Iterator
    pub fn observation_keys(&self) -> Keys<'_, ObsKey, Observation> {
        self.record.as_obs().into_iter().keys()
    }

    /// Returns [SignalObservation] Iterator
    pub fn signal_observations_iter(&self) -> Iter<'_, ObsKey, SignalObservation> {
        self.record
            .as_obs()
            .into_iter()
            .filter_map(|obs| obs.as_signal())
    }

    /// Returns mutable [ClockObservation] Iterator
    pub fn signal_observations_iter_mut(&mut self) -> IterMut<'_, ObsKey, SignalObservation> {
        self.record
            .as_obs_mut()
            .into_iter()
            .filter_map(|obs| obs.as_mut_signal())
    }

    /// Returns [ClockObservation] Iterator
    pub fn signal_observations_iter(&self) -> Iter<'_, ObsKey, SignalObservation> {
        self.record
            .as_obs()
            .into_iter()
            .filter_map(|obs| obs.as_signal())
    }

    /// Returns mutable [ClockObservation] Iterator
    pub fn signal_observations_iter_mut(&mut self) -> IterMut<'_, ObsKey, SignalObservation> {
        self.record
            .as_obs_mut()
            .into_iter()
            .filter_map(|obs| obs.as_mut_signal())
    }

    /// Returns mutable [SignalObservation] Iterator
    /// Returns [ClockObservation] Iterator
    /// Returns mutable [ClockObservation] Iterator

    /// Returns Observation record iterator. Unlike other records,
    /// an [`EpochFlag`] is attached to each individual [`Epoch`]
    /// to either validated or invalidate it.
    /// Clock receiver offset (in seconds), if present, are defined for each individual
    /// [`Epoch`].
    /// Phase data is exposed as raw / unscaled data: therefore incorrect
    /// values in case of High Precision RINEX. Prefer the dedicated
    /// [Self::carrier_phase] iterator. In any case, you should always
    /// prefer the iteration method of the type of data you're interested in.
    /// ```
    /// use rinex::prelude::*;
    /// use gnss_rs::prelude::SV;
    /// // macros
    /// use gnss_rs::sv;
    /// use rinex::observable;
    /// use std::str::FromStr; // observable!, sv!
    ///
    /// let rnx = Rinex::from_file("../test_resources/CRNX/V3/KUNZ00CZE.crx")
    ///    .unwrap();
    ///
    /// for ((epoch, flag), (clock_offset, vehicles)) in rnx.observation() {
    ///     assert!(flag.is_ok()); // no invalid epochs in this file
    ///     assert!(clock_offset.is_none()); // we don't have an example for this, at the moment
    ///     for (sv, observations) in vehicles {
    ///         if *sv == sv!("E01") {
    ///             for (observable, observation) in observations {
    ///                 if *observable == observable!("L1C") {
    ///                     if let Some(lli) = observation.lli {
    ///                         // A flag might be attached to each observation.
    ///                         // Implemented as `bitflag`, it supports bit masking operations
    ///                     }
    ///                     if let Some(snri) = observation.snr {
    ///                         // SNR indicator might exist too
    ///                     }
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn observation(
        &self,
    ) -> Box<
        dyn Iterator<
                Item = (
                    &(Epoch, EpochFlag),
                    &(
                        Option<f64>,
                        BTreeMap<SV, HashMap<Observable, ObservationData>>,
                    ),
                ),
            > + '_,
    > {
        Box::new(
            self.record
                .as_obs()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
}
