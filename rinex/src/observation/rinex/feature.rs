//! Feature dependent high level methods

use itertools::Itertools;

use crate::prelude::{Carrier, LliFlags, ObsKey, Rinex, SignalObservation};

#[cfg(docsrs)]
use crate::prelude::{Epoch, EpochFlag, Observable};

/// Supported signal [Combination]s
#[derive(Debug, Copy, Clone)]
pub enum Combination {
    /// Geometry Free (GF) combination
    GeometryFree,
    /// Ionosphere Free (IF) combination
    IonosphereFree,
    /// Wide Lane (Wl) combination
    WideLane,
    /// Narrow Lane (Nl) combination
    NarrowLane,
    /// Melbourne Wubbena (MW) combination
    MelbourneWubbena,
}

/// Definition of a [SignalCombination]
#[derive(Debug, Clone)]
pub struct SignalCombination {
    pub combination: Combination,
    /// LHS [SignalObservation] of the [Combination] (lhs - ref)
    pub lhs: SignalObservation,
    /// Reference [SignalObservation] used in [Combination] (lhs - ref)
    pub reference: SignalObservation,
}

// TODO
// #[cfg(feature = "obs")]
// use observation::Dcb;

// #[cfg(feature = "obs")]
// #[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
// impl Dcb for Rinex {
//     fn dcb(&self) -> HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
//         if let Some(r) = self.record.as_obs() {
//             r.dcb()
//         } else {
//             panic!("wrong rinex type");
//         }
//     }
// }

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

    // /// Iterator over all possible signal [Combination]s.
    // /// NB: we only combine pseudo range and / or phase range, not other observations.
    // /// You can then apply your own filter, to retain for example pseudo range combination only.
    // /// Some combinations will cross-mix physics.
    // pub fn signal_combinations_iter(&self) -> Iter<'_, ObsKey, SignalCombination> {
    //     self.observations_iter()
    // }

    // /// Iterator over specific signal [Combination].
    // /// NB: we only combine pseudo range and / or phase range, not other observations.
    // /// You can then apply your own filter, to retain for example pseudo range combination only.
    // /// Some combinations will cross-mix physics.
    // pub fn signal_combination_iter(&self, combination: Combination) -> Iter<'_, ObsKey, SignalCombination> {
    //     self.signal_combinations_iter()
    //         .filter_map(|(k, v))| {
    //             if v.combination == combination {
    //                 Some((k, v))
    //             } else {
    //                 None
    //             }
    //         })
    //     }
    // }
}

//     /// See [signal_combinations()]
//     pub fn pseudo_range_combinations(
//         &self,
//     ) -> Box<dyn Iterator<Item = (&Observable, &Observable)> + '_> {
//         Box::new(
//             self.observation()
//                 .flat_map(|(_, (_, svnn))| {
//                     svnn.iter().flat_map(|(_, obs)| {
//                         obs.iter().flat_map(|(lhs_ob, _)| {
//                             obs.iter().flat_map(move |(rhs_ob, _)| {
//                                 if lhs_ob.is_pseudorange_observable() && lhs_ob.same_physics(rhs_ob)
//                                 {
//                                     if lhs_ob != rhs_ob {
//                                         Some((lhs_ob, rhs_ob))
//                                     } else {
//                                         None
//                                     }
//                                 } else {
//                                     None
//                                 }
//                             })
//                         })
//                     })
//                 })
//                 .unique(),
//         )
//     }

//     /// See [signal_combinations()]
//     pub fn phase_range_combinations(
//         &self,
//     ) -> Box<dyn Iterator<Item = (&Observable, &Observable)> + '_> {
//         Box::new(
//             self.observation()
//                 .flat_map(|(_, (_, svnn))| {
//                     svnn.iter().flat_map(|(_, obs)| {
//                         obs.iter().flat_map(|(lhs_ob, _)| {
//                             obs.iter().flat_map(move |(rhs_ob, _)| {
//                                 if lhs_ob.is_phase_observable() && lhs_ob.same_physics(rhs_ob) {
//                                     if lhs_ob != rhs_ob {
//                                         Some((lhs_ob, rhs_ob))
//                                     } else {
//                                         None
//                                     }
//                                 } else {
//                                     None
//                                 }
//                             })
//                         })
//                     })
//                 })
//                 .unique(),
//         )
//     }

//     /// Returns an iterator over phase data, expressed in (whole) carrier cycles.
//     /// If Self is a High Precision RINEX (scaled RINEX), data is correctly scaled.
//     /// High precision RINEX allows up to 100 pico carrier cycle precision.
//     /// ```
//     /// use rinex::prelude::*;
//     /// use rinex::observable;
//     /// use std::str::FromStr;
//     ///
//     /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
//     ///     .unwrap();
//     /// // example: design a L1 signal iterator
//     /// let phase_l1c = rnx.carrier_phase()
//     ///     .filter_map(|(e, sv, obs, value)| {
//     ///         if *obs == observable!("L1C") {
//     ///             Some((e, sv, value))
//     ///         } else {
//     ///             None
//     ///         }
//     ///     });
//     /// ```
//     pub fn carrier_phase(
//         &self,
//     ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
//         Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
//             vehicles.iter().flat_map(|(sv, observations)| {
//                 observations.iter().filter_map(|(observable, obsdata)| {
//                     if observable.is_phase_observable() {
//                         if let Some(header) = &self.header.obs {
//                             // apply a scaling (if any), otherwise preserve data precision
//                             if let Some(scaling) =
//                                 header.scaling(sv.constellation, observable.clone())
//                             {
//                                 Some((*e, *sv, observable, obsdata.obs / *scaling as f64))
//                             } else {
//                                 Some((*e, *sv, observable, obsdata.obs))
//                             }
//                         } else {
//                             Some((*e, *sv, observable, obsdata.obs))
//                         }
//                     } else {
//                         None
//                     }
//                 })
//             })
//         }))
//     }
//     /// Returns an iterator over pseudo range observations.
//     /// ```
//     /// use rinex::prelude::*;
//     /// use rinex::observable;
//     /// use std::str::FromStr;
//     ///
//     /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
//     ///     .unwrap();
//     /// // example: design a C1 pseudo range iterator
//     /// let c1 = rnx.pseudo_range()
//     ///     .filter_map(|(e, sv, obs, value)| {
//     ///         if *obs == observable!("C1") {
//     ///             Some((e, sv, value))
//     ///         } else {
//     ///             None
//     ///         }
//     ///     });
//     /// ```
//     pub fn pseudo_range(
//         &self,
//     ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
//         Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
//             vehicles.iter().flat_map(|(sv, observations)| {
//                 observations.iter().filter_map(|(obs, obsdata)| {
//                     if obs.is_pseudorange_observable() {
//                         Some((*e, *sv, obs, obsdata.obs))
//                     } else {
//                         None
//                     }
//                 })
//             })
//         }))
//     }
//     /// Returns an Iterator over pseudo range observations in valid
//     /// Epochs, with valid LLI flags
//     pub fn pseudo_range_ok(&self) -> Box<dyn Iterator<Item = (Epoch, SV, &Observable, f64)> + '_> {
//         Box::new(self.observation().flat_map(|((e, flag), (_, vehicles))| {
//             vehicles.iter().flat_map(|(sv, observations)| {
//                 observations.iter().filter_map(|(obs, obsdata)| {
//                     if obs.is_pseudorange_observable() {
//                         if flag.is_ok() {
//                             Some((*e, *sv, obs, obsdata.obs))
//                         } else {
//                             None
//                         }
//                     } else {
//                         None
//                     }
//                 })
//             })
//         }))
//     }

//     /// Returns an Iterator over fractional pseudo range observations
//     pub fn pseudo_range_fract(
//         &self,
//     ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
//         Box::new(self.pseudo_range().filter_map(|(e, sv, observable, pr)| {
//             if let Some(t) = observable.code_length(sv.constellation) {
//                 let c = 299792458_f64; // speed of light
//                 Some((e, sv, observable, pr / c / t))
//             } else {
//                 None
//             }
//         }))
//     }
//     /// Returns an iterator over doppler shifts. A positive doppler
//     /// means SV is moving towards receiver.
//     /// ```
//     /// use rinex::prelude::*;
//     /// use rinex::observable;
//     /// use std::str::FromStr;
//     ///
//     /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
//     ///     .unwrap();
//     /// // example: design a L1 signal doppler iterator
//     /// let doppler_l1 = rnx.doppler()
//     ///     .filter_map(|(e, sv, obs, value)| {
//     ///         if *obs == observable!("D1") {
//     ///             Some((e, sv, value))
//     ///         } else {
//     ///             None
//     ///         }
//     ///     });
//     /// ```
//     pub fn doppler(
//         &self,
//     ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
//         Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
//             vehicles.iter().flat_map(|(sv, observations)| {
//                 observations.iter().filter_map(|(obs, obsdata)| {
//                     if obs.is_doppler_observable() {
//                         Some((*e, *sv, obs, obsdata.obs))
//                     } else {
//                         None
//                     }
//                 })
//             })
//         }))
//     }
//     /// Returns an iterator over signal strength observations.
//     /// ```
//     /// use rinex::prelude::*;
//     /// use rinex::observable;
//     /// use std::str::FromStr;
//     ///
//     /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
//     ///     .unwrap();
//     /// // example: design a S1: L1 strength iterator
//     /// let ssi_l1 = rnx.ssi()
//     ///     .filter_map(|(e, sv, obs, value)| {
//     ///         if *obs == observable!("S1") {
//     ///             Some((e, sv, value))
//     ///         } else {
//     ///             None
//     ///         }
//     ///     });
//     /// ```
//     pub fn ssi(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
//         Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
//             vehicles.iter().flat_map(|(sv, observations)| {
//                 observations.iter().filter_map(|(obs, obsdata)| {
//                     if obs.is_ssi_observable() {
//                         Some((*e, *sv, obs, obsdata.obs))
//                     } else {
//                         None
//                     }
//                 })
//             })
//         }))
//     }
//     /// Returns an Iterator over signal SNR indications.
//     /// All observation that did not come with such indication are filtered out.
//     /// ```
//     /// use rinex::*;
//     /// let rinex =
//     ///     Rinex::from_file("../test_resources/OBS/V3/ALAC00ESP_R_20220090000_01D_30S_MO.rnx")
//     ///         .unwrap();
//     /// for ((e, flag), sv, observable, snr) in rinex.snr() {
//     ///     // See RINEX specs or [SNR] documentation
//     ///     if snr.weak() {
//     ///     } else if snr.strong() {
//     ///     } else if snr.excellent() {
//     ///     }
//     ///     // you can directly compare to dBHz
//     ///     if snr < 29.0.into() {
//     ///         // considered weak signal
//     ///     } else if snr >= 30.0.into() {
//     ///         // considered strong signal
//     ///     }
//     /// }
//     /// ```
//     pub fn snr(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, SNR)> + '_> {
//         Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
//             vehicles.iter().flat_map(|(sv, observations)| {
//                 observations
//                     .iter()
//                     .filter_map(|(obs, obsdata)| obsdata.snr.map(|snr| (*e, *sv, obs, snr)))
//             })
//         }))
//     }
//     /// Returns an Iterator over LLI flags that might be associated to an Observation.
//     /// ```
//     /// use rinex::*;
//     /// use rinex::observation::LliFlags;
//     /// let rinex =
//     ///     Rinex::from_file("../test_resources/OBS/V3/ALAC00ESP_R_20220090000_01D_30S_MO.rnx")
//     ///         .unwrap();
//     /// let custom_mask
//     ///     = LliFlags::OK_OR_UNKNOWN | LliFlags::UNDER_ANTI_SPOOFING;
//     /// for ((e, flag), sv, observable, lli) in rinex.lli() {
//     ///     // See RINEX specs or [LliFlags] documentation
//     ///     if lli.intersects(custom_mask) {
//     ///         // sane observation but under AS
//     ///     }
//     /// }
//     /// ```
//     pub fn lli(
//         &self,
//     ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, LliFlags)> + '_> {
//         Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
//             vehicles.iter().flat_map(|(sv, observations)| {
//                 observations
//                     .iter()
//                     .filter_map(|(obs, obsdata)| obsdata.lli.map(|lli| (*e, *sv, obs, lli)))
//             })
//         }))
//     }
//     /// Returns an Iterator over "complete" Epochs.
//     /// "Complete" Epochs are Epochs were both Phase and Pseudo Range
//     /// observations are present on two carriers, sane sampling conditions are met
//     /// and an optional minimal SNR criteria is met (disregarded if None).
//     pub fn complete_epoch(
//         &self,
//         min_snr: Option<SNR>,
//     ) -> Box<dyn Iterator<Item = (Epoch, Vec<(SV, Carrier)>)> + '_> {
//         Box::new(
//             self.observation()
//                 .filter_map(move |((e, flag), (_, vehicles))| {
//                     if flag.is_ok() {
//                         let mut list: Vec<(SV, Carrier)> = Vec::new();
//                         for (sv, observables) in vehicles {
//                             let mut l1_pr_ph = (false, false);
//                             let mut lx_pr_ph: HashMap<Carrier, (bool, bool)> = HashMap::new();
//                             for (observable, observation) in observables {
//                                 if !observable.is_phase_observable()
//                                     && !observable.is_pseudorange_observable()
//                                 {
//                                     continue; // not interesting here
//                                 }
//                                 let carrier =
//                                     Carrier::from_observable(sv.constellation, observable);
//                                 if carrier.is_err() {
//                                     // fail to identify this signal
//                                     continue;
//                                 }
//                                 if let Some(min_snr) = min_snr {
//                                     if let Some(snr) = observation.snr {
//                                         if snr < min_snr {
//                                             continue;
//                                         }
//                                     } else {
//                                         continue; // can't compare to criteria
//                                     }
//                                 }
//                                 let carrier = carrier.unwrap();
//                                 if carrier == Carrier::L1 {
//                                     l1_pr_ph.0 |= observable.is_pseudorange_observable();
//                                     l1_pr_ph.1 |= observable.is_phase_observable();
//                                 } else if let Some((lx_pr, lx_ph)) = lx_pr_ph.get_mut(&carrier) {
//                                     *lx_pr |= observable.is_pseudorange_observable();
//                                     *lx_ph |= observable.is_phase_observable();
//                                 } else if observable.is_pseudorange_observable() {
//                                     lx_pr_ph.insert(carrier, (true, false));
//                                 } else if observable.is_phase_observable() {
//                                     lx_pr_ph.insert(carrier, (false, true));
//                                 }
//                             }
//                             if l1_pr_ph == (true, true) {
//                                 for (carrier, (pr, ph)) in lx_pr_ph {
//                                     if pr && ph {
//                                         list.push((*sv, carrier));
//                                     }
//                                 }
//                             }
//                         }
//                         Some((*e, list))
//                     } else {
//                         None
//                     }
//                 })
//                 .filter(|(_sv, list)| !list.is_empty()),
//         )
//     }
//     /// Returns Code Multipath bias estimates, for sampled code combination and per SV.
//     /// Refer to [Bibliography::ESABookVol1] and [Bibliography::MpTaoglas].
//     pub fn code_multipath(
//         &self,
//     ) -> HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
//         if let Some(r) = self.record.as_obs() {
//             code_multipath(r)
//         } else {
//             HashMap::new()
//         }
//     }
