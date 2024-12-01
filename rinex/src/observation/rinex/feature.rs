//! Feature dependent high level methods

use itertools::Itertools;

use crate::prelude::{Carrier, LliFlags, ObsKey, Observable, Rinex, SignalObservation};

#[cfg(docsrs)]
use crate::prelude::{Epoch, EpochFlag};

/// Supported signal [Combination]s
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Combination {
    /// Geometry Free (GF) combination (same physics)
    GeometryFree,
    /// Ionosphere Free (IF) combination (same physics)
    IonosphereFree,
    /// Wide Lane (Wl) combination (same physics)
    WideLane,
    /// Narrow Lane (Nl) combination (same physics)
    NarrowLane,
    /// Melbourne-Wübbena (MW) combination (cross-mixed physics)
    MelbourneWubbena,
}

/// Definition of a [SignalCombination]
#[derive(Debug, Clone)]
pub struct SignalCombination {
    /// [Combination] that was formed
    pub combination: Combination,
    /// Reference [Observable]
    pub reference: Observable,
    /// Left hand side (compared) [Observable]
    pub lhs: Observable,
    /// Value, unit is meters of delay of the (lhs - reference) frequency
    pub value: f64,
}

impl Rinex {
    /// Returns [Carrier] signals Iterator
    pub fn carrier_iter(&self) -> Box<dyn Iterator<Item = Carrier> + '_> {
        Box::new(
            self.signal_observations_iter()
                .filter_map(|(_, sig)| {
                    if let Ok(carrier) =
                        Carrier::from_observable(sig.sv.constellation, &sig.observable)
                    {
                        Some(carrier)
                    } else {
                        None
                    }
                })
                .unique(),
        )
    }

    /// Returns Signal Code Iterator (official RINEX denomination).
    /// For example, "1C" means L1 civilian, or "1P" means L1 precision code.
    pub fn signal_code_iter(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(
            self.signal_observations_iter()
                .filter_map(|(_, sig)| {
                    let code = sig.observable.code()?;
                    Some(code)
                })
                .unique(),
        )
    }

    /// [SignalObservation]s Iterator for which sampling conditions were marked OK.
    /// For [Observable::PhaseRange] you should verify the tracking status as well, for completeness.
    /// You can use:
    /// - [Self::pseudo_range_sampling_ok_iter()] if you're only interested in decoded pseudo range
    /// - [Self::phase_range_sampling_ok_iter()] if you're only interested in estimated phase range
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    ///
    /// for (key, signal) in rinex.signal_ok_iter() {
    ///     let t = key.epoch;
    ///     let flag = key.flag;
    ///     let sv = signal.sv; // signal source
    ///     assert!(flag.is_ok(), "all abnormal flags filtered out");
    ///     match signal.observable {
    ///         Observable::PhaseRange(pr) => {
    ///             // then do something
    ///         },
    ///         Observable::PseudoRange(pr) => {
    ///             // then do something
    ///         },
    ///         Observable::Doppler(dop) => {
    ///             // then do something
    ///         },
    ///         Observable::SSI(ssi) => {
    ///             // then do something
    ///         },
    ///     }
    /// }
    /// ```
    pub fn signal_observations_sampling_ok_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (ObsKey, &SignalObservation)> + '_> {
        Box::new(self.signal_observations_iter().filter_map(|(k, sig)| {
            if !k.flag.is_ok() {
                Some((k, sig))
            } else {
                None
            }
        }))
    }

    /// [Self::signals_observations_sampling_ok_iter()] with [Observable::PseudoRange] mask applied.
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    ///
    /// for (key, signal) in rinex.pseudo_range_decoding_ok_iter() {
    ///     let t = key.epoch;
    ///     let flag = key.flag;
    ///     assert!(flag.is_ok(), "all abnormal flags filtered out");
    ///
    ///     let sv = signal.sv; // signal source
    ///     // RINEX decodes [Observable::PseudoRange] to meters
    ///     let pseudo_range_m = signal.value;
    /// }
    /// ```
    pub fn pseudo_range_sampling_ok_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (ObsKey, &SignalObservation)> + '_> {
        Box::new(
            self.signal_observations_sampling_ok_iter()
                .filter_map(|(k, sig)| {
                    if sig.observable.is_pseudo_range_observable() {
                        Some((k, sig))
                    } else {
                        None
                    }
                }),
        )
    }

    /// Returns an Iterator over [Epoch]s where [Observable::PhaseRange] sampling took place in good conditions.
    /// For good tracking conditions, you need to take the attached [LliFlags] into account.
    /// See [Rinex::signal_ok_iter()] for more information.
    /// Use [Rinex::phase_range_tracking_ok_iter()] for both good sampling and tracking conditions filtering.
    /// Use [Rinex::phase_tracking_issues_iter()] for tracking errors iteration.
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    ///
    /// for (key, signal) in rinex.phase_range_sampling_ok_iter() {
    ///     let t = key.epoch;
    ///     let flag = key.flag;
    ///     assert!(flag.is_ok(), "all abnormal flags filtered out");
    ///
    ///     let sv = signal.sv; // signal source
    ///     // RINEX measures [Observable::PhaseRange] in meters directly
    ///     let phase_range_m = signal.value;
    /// }
    /// ```
    pub fn phase_range_sampling_ok_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (ObsKey, &SignalObservation)> + '_> {
        Box::new(
            self.signal_observations_sampling_ok_iter()
                .filter_map(|(k, sig)| {
                    if sig.observable.is_phase_range_observable() {
                        Some((k, sig))
                    } else {
                        None
                    }
                }),
        )
    }

    /// Returns an Iterator over [Epoch]s where [Observable::PhaseRange] sampling and tracking took place in good conditions.
    /// See [Rinex::signal_ok_iter()] for more information.
    /// Use [Rinex::phase_range_sampling_ok_iter()] for testing only the sampling conditions.
    /// Use [Rinex::phase_tracking_issues_iter()] for anomalies analysis.
    /// NB: if [LliFlags] is missing, we consider the tracking was GOOD. Because some receivers omit or do not encode
    /// the [LliFlags] and would rather not stream out RINEX in such situation (and we would wind up with empty dataset).
    /// Read the secondary example down below.
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    ///
    /// for (key, signal) in rinex.phase_range_tracking_ok_iter() {
    ///     let t = key.epoch;
    ///     let flag = key.flag;
    ///     assert!(flag.is_ok(), "all abnormal flags filtered out");
    ///
    ///     let sv = signal.sv; // signal source
    ///     // RINEX measures [Observable::PhaseRange] in meters directly
    ///     let phase_range_m = signal.value;
    /// }
    /// ```
    ///
    /// You can design your own iterator to make sure [LliFlags] is present and tracking was definitely good:
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    ///
    /// // retain only observations where [LliFlags] are present and sane.
    /// // You could even stack a minimal SNR.
    /// let tracking_all_good = rinex.phase_range_tracking_ok_iter()
    ///     .filter_map(|(k, v)| {
    ///         if let Some(lli) = v.lli {
    ///             if lli.is_ok() {
    ///                 Some((k, v))
    ///             } else {
    ///                 None
    ///             }
    ///         } else {
    ///             None
    ///         }
    ///     });
    ///
    /// for (key, signal) in rinex.tracking_all_good() {
    ///     let t = key.epoch;
    ///     let flag = key.flag;
    ///     assert!(flag.is_ok(), "all abnormal flags filtered out");
    ///
    ///     let sv = signal.sv; // signal source
    ///     // RINEX measures [Observable::PhaseRange] in meters directly
    ///     let phase_range_m = signal.value;
    /// }
    /// ```
    pub fn phase_range_tracking_ok_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (ObsKey, &SignalObservation)> + '_> {
        Box::new(self.phase_range_sampling_ok_iter().filter_map(|(k, sig)| {
            if let Some(lli) = sig.lli {
                if lli.intersects(LliFlags::OK_OR_UNKNOWN) {
                    Some((k, sig))
                } else {
                    None
                }
            } else {
                Some((k, sig))
            }
        }))
    }

    /// Returns Iterator over Phase Cycle slips events.
    ///```
    /// use rinex::prelude::{Rinex, LliFlags};
    ///
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    ///
    /// for (key, signal) in rinex.phase_cycle_slip_events() {
    ///     let t = key.epoch;
    ///     let sv = signal.sv; // signal source
    ///     let lli = signal.lli.unwrap(); // you're safe at this point
    ///     assert!(lli.intersects(LliFlags::LOCK_LOSS));
    /// }
    /// ```
    pub fn phase_cycle_slip_events(
        &self,
    ) -> Box<dyn Iterator<Item = (ObsKey, &SignalObservation)> + '_> {
        Box::new(self.signal_observations_iter().filter_map(|(k, sig)| {
            if sig.observable.is_phase_range_observable() {
                if let Some(lli) = sig.lli {
                    if lli.intersects(LliFlags::LOCK_LOSS) {
                        Some((k, sig))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }))
    }

    /// Returns Iterator over both Half and Full Phase Cycle slips events.
    /// Use provided [LliFlags] to inquire.
    ///```
    /// use rinex::prelude::{Rinex, LliFlags};
    ///
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    ///
    /// for (key, signal) in rinex.phase_half_full_cycle_slip_events() {
    ///     let t = key.epoch;
    ///     let sv = signal.sv; // signal source
    ///     let lli = signal.lli.unwrap(); // you're safe at this point
    ///     let is_half = lli.intersects(LliFlags::HALF_CYCLE_SLIP);
    ///     if is_half {
    ///         // do something
    ///     } else {
    ///         // then, do something
    ///     }
    /// }
    /// ```
    pub fn phase_half_full_cycle_slip_events(
        &self,
    ) -> Box<dyn Iterator<Item = (ObsKey, &SignalObservation)> + '_> {
        Box::new(self.phase_range_observations_iter().filter_map(|(k, sig)| {
            let lli = sig.lli?;
            if lli.intersects(LliFlags::LOCK_LOSS) || lli.intersects(LliFlags::HALF_CYCLE_SLIP) {
                Some((k, sig))
            } else {
                None
            }
        }))
    }

    /// Copies and returns a new [Rinex] where given [LliFlags] mask condition is met
    /// for all Phase data points (since [LliFlags] is only relevant to phase tracking).
    /// Use this to modify your dataset.
    pub fn phase_tracking_lli_and_mask(&self, and: LliFlags) -> Self {
        let mut s = self.clone();
        s.phase_tracking_mut_lli_and_mask(and);
        s
    }

    /// Mutable [Self::phase_tracking_lli_and_mask] implementation.
    /// Use this to modify your dataset in place.
    pub fn phase_tracking_mut_lli_and_mask(&mut self, and: LliFlags) {
        if let Some(rec) = self.record.as_mut_obs() {
            rec.retain(|_, v| {
                v.signals.retain(|sig| {
                    if sig.observable.is_phase_range_observable() {
                        if let Some(lli) = sig.lli {
                            lli.intersects(and)
                        } else {
                            false
                        }
                    } else {
                        true
                    }
                });
                !v.signals.is_empty()
            })
        }
    }

    /// Returns an Iterator over [Epoch]s where abnormal sampling conditions were detected.
    /// Anomalies are described by the attached [EpochFlag] in each [ObsKey].
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    ///
    /// let anomalies = rinex.epoch_anomalies().collect::<Vec<_>>();
    /// assert_eq!(anomalies.len(), 0, "no anomalies detected");
    /// ```
    pub fn epoch_anomalies(&self) -> Box<dyn Iterator<Item = &ObsKey> + '_> {
        Box::new(self.observation_keys().filter(|k| k.flag.is_ok()))
    }

    // Design [SignalCombination]s iterator.
    pub fn signal_combinations_iter(
        &self,
        combination: Combination,
    ) -> Box<dyn Iterator<Item = (ObsKey, SignalCombination)> + '_> {
        if combination == Combination::MelbourneWubbena {
            // this is a cross-mixed combination

            let phase = self
                .signal_combinations_iter(Combination::WideLane)
                .filter(|(_, comb)| comb.reference.is_phase_range_observable());

            let code = self
                .signal_combinations_iter(Combination::NarrowLane)
                .filter(|(_, comb)| comb.reference.is_pseudo_range_observable());

            Box::new(phase.zip(code).map(move |((k, phase), (_, code))| {
                (
                    k,
                    SignalCombination {
                        combination,
                        reference: phase.reference.clone(),
                        lhs: phase.lhs.clone(),
                        value: phase.value - code.value,
                    },
                )
            }))
        } else {
            Box::new(
                self.signal_observations_iter()
                    .zip(self.signal_observations_iter())
                    .filter_map(move |((k_1, sig_1), (k_2, sig_2))| {
                        // combine only synchronous observations
                        let synchronous = k_1.epoch == k_2.epoch;

                        // only same physics combinations at this point
                        let same_physics = sig_1.observable.same_physics(&sig_2.observable);

                        // consider only L1 on left hand iterator
                        let sv_1 = sig_1.sv;
                        let code_1 = sig_1.observable.to_string();
                        let lhs_is_l1 = code_1.contains('1');
                        let carrier_1 = sig_1.observable.carrier(sv_1.constellation);

                        // consider anything but L1 on right hand iterator
                        let sv_2 = sig_2.sv;
                        let code_2 = sig_2.observable.to_string();
                        let rhs_is_lj = !code_2.contains('1');
                        let carrier_2 = sig_1.observable.carrier(sv_2.constellation);

                        // need correct frequency interpretation on both sides
                        let carriers_ok = carrier_1.is_ok() && carrier_2.is_ok();

                        if synchronous && same_physics && lhs_is_l1 && rhs_is_lj && carriers_ok {
                            let f_1 = carrier_1.unwrap().frequency();
                            let f_2 = carrier_2.unwrap().frequency();

                            let alpha = match combination {
                                Combination::GeometryFree => 1.0,
                                Combination::IonosphereFree => f_1.powi(2),
                                Combination::WideLane => f_1,
                                Combination::NarrowLane => f_1,
                                Combination::MelbourneWubbena => {
                                    unreachable!("mw combination");
                                },
                            };

                            let beta = match combination {
                                Combination::GeometryFree => -1.0,
                                Combination::IonosphereFree => -f_2.powi(2),
                                Combination::WideLane => -f_2,
                                Combination::NarrowLane => f_2,
                                Combination::MelbourneWubbena => {
                                    unreachable!("mw combination");
                                },
                            };

                            let gamma = match combination {
                                Combination::GeometryFree => 1.0,
                                Combination::IonosphereFree => (f_1.powi(2) - f_2.powi(2)),
                                Combination::WideLane => f_1 - f_2,
                                Combination::NarrowLane => f_1 + f_2,
                                Combination::MelbourneWubbena => {
                                    unreachable!("mw combination");
                                },
                            };

                            let (v_lhs, v_rhs) = match combination {
                                Combination::GeometryFree => {
                                    if sig_1.observable.is_pseudo_range_observable() {
                                        (sig_2.value, sig_1.value)
                                    } else {
                                        (sig_1.value, sig_2.value)
                                    }
                                },
                                Combination::IonosphereFree => (sig_1.value, sig_2.value),
                                Combination::WideLane => (sig_1.value, sig_2.value),
                                Combination::NarrowLane => (sig_1.value, sig_2.value),
                                Combination::MelbourneWubbena => {
                                    unreachable!("mw combination");
                                },
                            };

                            Some((
                                k_1,
                                SignalCombination {
                                    combination,
                                    lhs: sig_1.observable.clone(),
                                    reference: sig_2.observable.clone(),
                                    value: (alpha * v_lhs + beta * v_rhs) / gamma,
                                },
                            ))
                        } else {
                            None
                        }
                    }),
            )
        }
    }
}
