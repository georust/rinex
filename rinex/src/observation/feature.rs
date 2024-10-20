//! Feature dependent high level methods

use crate::prelude::{Carrier, ObsKey, Rinex};

#[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
impl Rinex {
    /// Returns a Unique Iterator over identified [Carrier]s
    pub fn carrier_iter(&self) -> Box<dyn Iterator<Item = Carrier> + '_> {
        Box::new(
            self.observation()
                .flat_map(|(_, (_, sv))| {
                    sv.iter().flat_map(|(sv, observations)| {
                        observations.keys().filter_map(|observable| {
                            Some(observable.carrier(sv.constellation).ok()?)
                        })
                    })
                })
                .unique(),
        )
    }
    /// Returns a Unique Iterator over signal Codes, like "1C" or "1P"
    /// for precision code.
    pub fn code(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(
            self.observation()
                .flat_map(|(_, (_, sv))| {
                    sv.iter().flat_map(|(_, observations)| {
                        observations
                            .keys()
                            .filter_map(|observable| observable.code())
                    })
                })
                .unique(),
        )
    }
    /// Returns Unique Iterator over all feasible Pseudo range and Phase range combination,
    /// expressed as (lhs: Observable, rhs: Observable).
    /// Regardless which one is to consider as reference signal.
    /// Use [pseudo_range_combinations()] or [phase_range_combinations()]
    /// to reduce to specific physical observations.
    pub fn signal_combinations(&self) -> Box<dyn Iterator<Item = (&Observable, &Observable)> + '_> {
        Box::new(
            self.pseudo_range_combinations()
                .chain(self.phase_range_combinations()),
        )
    }
    /// See [signal_combinations()]
    pub fn pseudo_range_combinations(
        &self,
    ) -> Box<dyn Iterator<Item = (&Observable, &Observable)> + '_> {
        Box::new(
            self.observation()
                .flat_map(|(_, (_, svnn))| {
                    svnn.iter().flat_map(|(_, obs)| {
                        obs.iter().flat_map(|(lhs_ob, _)| {
                            obs.iter().flat_map(move |(rhs_ob, _)| {
                                if lhs_ob.is_pseudorange_observable() && lhs_ob.same_physics(rhs_ob)
                                {
                                    if lhs_ob != rhs_ob {
                                        Some((lhs_ob, rhs_ob))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                        })
                    })
                })
                .unique(),
        )
    }
    /// See [signal_combinations()]
    pub fn phase_range_combinations(
        &self,
    ) -> Box<dyn Iterator<Item = (&Observable, &Observable)> + '_> {
        Box::new(
            self.observation()
                .flat_map(|(_, (_, svnn))| {
                    svnn.iter().flat_map(|(_, obs)| {
                        obs.iter().flat_map(|(lhs_ob, _)| {
                            obs.iter().flat_map(move |(rhs_ob, _)| {
                                if lhs_ob.is_phase_observable() && lhs_ob.same_physics(rhs_ob) {
                                    if lhs_ob != rhs_ob {
                                        Some((lhs_ob, rhs_ob))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                        })
                    })
                })
                .unique(),
        )
    }
    /// Returns ([`Epoch`] [`EpochFlag`]) iterator, where each {`EpochFlag`]
    /// validates or invalidates related [`Epoch`]
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// for (epoch, flag) in rnx.epoch_flag() {
    ///     assert!(flag.is_ok()); // no invalid epoch
    /// }
    /// ```
    pub fn epoch_flag(&self) -> Box<dyn Iterator<Item = (Epoch, EpochFlag)> + '_> {
        Box::new(self.observation().map(|(e, _)| *e))
    }
    /// Returns an Iterator over all abnormal [`Epoch`]s
    /// and reports given event nature.  
    /// Refer to [`epoch::EpochFlag`] for all possible events.  
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// ```
    pub fn epoch_anomalies(&self) -> Box<dyn Iterator<Item = (Epoch, EpochFlag)> + '_> {
        Box::new(self.epoch_flag().filter_map(
            |(e, f)| {
                if !f.is_ok() {
                    Some((e, f))
                } else {
                    None
                }
            },
        ))
    }
    /// Returns an iterator over all [`Epoch`]s that have
    /// an [`EpochFlag::Ok`] flag attached to them
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// ```
    pub fn epoch_ok(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(
            self.epoch_flag()
                .filter_map(|(e, f)| if f.is_ok() { Some(e) } else { None }),
        )
    }
    /// Returns an iterator over all [`Epoch`]s where
    /// a Cycle Slip is declared by the receiver
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// ```
    pub fn epoch_cs(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(self.epoch_flag().filter_map(|(e, f)| {
            if f == EpochFlag::CycleSlip {
                Some(e)
            } else {
                None
            }
        }))
    }
    /// Returns an iterator over receiver clock offsets, expressed in seconds.
    /// Such information is kind of rare (modern / dual frequency receivers?)
    /// and we don't have a compelling example yet.
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// for ((epoch, flag), clk) in rnx.recvr_clock() {
    ///     // epoch: [hifitime::Epoch]
    ///     // clk: receiver clock offset [s]
    /// }
    /// ```
    pub fn recvr_clock(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), f64)> + '_> {
        Box::new(
            self.observation()
                .filter_map(|(e, (clk, _))| clk.as_ref().map(|clk| (*e, *clk))),
        )
    }
    /// Returns an iterator over phase data, expressed in (whole) carrier cycles.
    /// If Self is a High Precision RINEX (scaled RINEX), data is correctly scaled.
    /// High precision RINEX allows up to 100 pico carrier cycle precision.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a L1 signal iterator
    /// let phase_l1c = rnx.carrier_phase()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("L1C") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn carrier_phase(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(observable, obsdata)| {
                    if observable.is_phase_observable() {
                        if let Some(header) = &self.header.obs {
                            // apply a scaling (if any), otherwise preserve data precision
                            if let Some(scaling) =
                                header.scaling(sv.constellation, observable.clone())
                            {
                                Some((*e, *sv, observable, obsdata.obs / *scaling as f64))
                            } else {
                                Some((*e, *sv, observable, obsdata.obs))
                            }
                        } else {
                            Some((*e, *sv, observable, obsdata.obs))
                        }
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an iterator over pseudo range observations.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a C1 pseudo range iterator
    /// let c1 = rnx.pseudo_range()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("C1") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn pseudo_range(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_pseudorange_observable() {
                        Some((*e, *sv, obs, obsdata.obs))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an Iterator over pseudo range observations in valid
    /// Epochs, with valid LLI flags
    pub fn pseudo_range_ok(&self) -> Box<dyn Iterator<Item = (Epoch, SV, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|((e, flag), (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_pseudorange_observable() {
                        if flag.is_ok() {
                            Some((*e, *sv, obs, obsdata.obs))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
        }))
    }

    /// Returns an Iterator over fractional pseudo range observations
    pub fn pseudo_range_fract(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
        Box::new(self.pseudo_range().filter_map(|(e, sv, observable, pr)| {
            if let Some(t) = observable.code_length(sv.constellation) {
                let c = 299792458_f64; // speed of light
                Some((e, sv, observable, pr / c / t))
            } else {
                None
            }
        }))
    }
    /// Returns an iterator over doppler shifts. A positive doppler
    /// means SV is moving towards receiver.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a L1 signal doppler iterator
    /// let doppler_l1 = rnx.doppler()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("D1") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn doppler(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_doppler_observable() {
                        Some((*e, *sv, obs, obsdata.obs))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an iterator over signal strength observations.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a S1: L1 strength iterator
    /// let ssi_l1 = rnx.ssi()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("S1") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn ssi(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_ssi_observable() {
                        Some((*e, *sv, obs, obsdata.obs))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an Iterator over signal SNR indications.
    /// All observation that did not come with such indication are filtered out.
    /// ```
    /// use rinex::*;
    /// let rinex =
    ///     Rinex::from_file("../test_resources/OBS/V3/ALAC00ESP_R_20220090000_01D_30S_MO.rnx")
    ///         .unwrap();
    /// for ((e, flag), sv, observable, snr) in rinex.snr() {
    ///     // See RINEX specs or [SNR] documentation
    ///     if snr.weak() {
    ///     } else if snr.strong() {
    ///     } else if snr.excellent() {
    ///     }
    ///     // you can directly compare to dBHz
    ///     if snr < 29.0.into() {
    ///         // considered weak signal
    ///     } else if snr >= 30.0.into() {
    ///         // considered strong signal
    ///     }
    /// }
    /// ```
    pub fn snr(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, SNR)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations
                    .iter()
                    .filter_map(|(obs, obsdata)| obsdata.snr.map(|snr| (*e, *sv, obs, snr)))
            })
        }))
    }
    /// Returns an Iterator over LLI flags that might be associated to an Observation.
    /// ```
    /// use rinex::*;
    /// use rinex::observation::LliFlags;
    /// let rinex =
    ///     Rinex::from_file("../test_resources/OBS/V3/ALAC00ESP_R_20220090000_01D_30S_MO.rnx")
    ///         .unwrap();
    /// let custom_mask
    ///     = LliFlags::OK_OR_UNKNOWN | LliFlags::UNDER_ANTI_SPOOFING;
    /// for ((e, flag), sv, observable, lli) in rinex.lli() {
    ///     // See RINEX specs or [LliFlags] documentation
    ///     if lli.intersects(custom_mask) {
    ///         // sane observation but under AS
    ///     }
    /// }
    /// ```
    pub fn lli(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), SV, &Observable, LliFlags)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations
                    .iter()
                    .filter_map(|(obs, obsdata)| obsdata.lli.map(|lli| (*e, *sv, obs, lli)))
            })
        }))
    }
    /// Returns an Iterator over "complete" Epochs.
    /// "Complete" Epochs are Epochs were both Phase and Pseudo Range
    /// observations are present on two carriers, sane sampling conditions are met
    /// and an optional minimal SNR criteria is met (disregarded if None).
    pub fn complete_epoch(
        &self,
        min_snr: Option<SNR>,
    ) -> Box<dyn Iterator<Item = (Epoch, Vec<(SV, Carrier)>)> + '_> {
        Box::new(
            self.observation()
                .filter_map(move |((e, flag), (_, vehicles))| {
                    if flag.is_ok() {
                        let mut list: Vec<(SV, Carrier)> = Vec::new();
                        for (sv, observables) in vehicles {
                            let mut l1_pr_ph = (false, false);
                            let mut lx_pr_ph: HashMap<Carrier, (bool, bool)> = HashMap::new();
                            for (observable, observation) in observables {
                                if !observable.is_phase_observable()
                                    && !observable.is_pseudorange_observable()
                                {
                                    continue; // not interesting here
                                }
                                let carrier =
                                    Carrier::from_observable(sv.constellation, observable);
                                if carrier.is_err() {
                                    // fail to identify this signal
                                    continue;
                                }
                                if let Some(min_snr) = min_snr {
                                    if let Some(snr) = observation.snr {
                                        if snr < min_snr {
                                            continue;
                                        }
                                    } else {
                                        continue; // can't compare to criteria
                                    }
                                }
                                let carrier = carrier.unwrap();
                                if carrier == Carrier::L1 {
                                    l1_pr_ph.0 |= observable.is_pseudorange_observable();
                                    l1_pr_ph.1 |= observable.is_phase_observable();
                                } else if let Some((lx_pr, lx_ph)) = lx_pr_ph.get_mut(&carrier) {
                                    *lx_pr |= observable.is_pseudorange_observable();
                                    *lx_ph |= observable.is_phase_observable();
                                } else if observable.is_pseudorange_observable() {
                                    lx_pr_ph.insert(carrier, (true, false));
                                } else if observable.is_phase_observable() {
                                    lx_pr_ph.insert(carrier, (false, true));
                                }
                            }
                            if l1_pr_ph == (true, true) {
                                for (carrier, (pr, ph)) in lx_pr_ph {
                                    if pr && ph {
                                        list.push((*sv, carrier));
                                    }
                                }
                            }
                        }
                        Some((*e, list))
                    } else {
                        None
                    }
                })
                .filter(|(_sv, list)| !list.is_empty()),
        )
    }
    /// Returns Code Multipath bias estimates, for sampled code combination and per SV.
    /// Refer to [Bibliography::ESABookVol1] and [Bibliography::MpTaoglas].
    pub fn code_multipath(
        &self,
    ) -> HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
        if let Some(r) = self.record.as_obs() {
            code_multipath(r)
        } else {
            HashMap::new()
        }
    }
}
