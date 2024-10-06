use thiserror::Error;

use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use itertools::Itertools;

use crate::{
    epoch::{
        format as format_epoch, parse_in_timescale as parse_epoch_in_timescale,
        ParsingError as EpochParsingError,
    },
    merge::{Error as MergeError, Merge},
    observation::{Observation, SignalObservation, flag::Error as FlagError, EpochFlag, SNR, LliFlags},
    prelude::{Epoch, Header, SV, Duration, Constellation},
    split::{Error as SplitError, Split},
    types::Type,
    Carrier, Observable,
};

#[cfg(feature = "processing")]
use qc_traits::processing::{
    DecimationFilter, DecimationFilterType, FilterItem, MaskFilter, MaskOperand, Repair,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse epoch flag")]
    EpochFlag(#[from] FlagError),
    #[error("failed to parse epoch")]
    EpochError(#[from] EpochParsingError),
    #[error("constellation parsing error")]
    ConstellationParsing(#[from] gnss::constellation::ParsingError),
    #[error("sv parsing error")]
    SvParsing(#[from] gnss::sv::ParsingError),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse vehicles properly (nb_sat mismatch)")]
    EpochParsingError,
    #[error("line is empty")]
    MissingData,
    #[error("missing observable definitions")]
    MissingObservableSpecs,
}

/// Epoch (record entry) formatter
/// ## Inputs
///    - header: Heavily [Header] dependent
///    - epoch: Ongoing [Epoch]
///    - flag: Ongoing [EpochFlag]
///    - clk_offset: possible Clock Offset [s]
///    - signals: [SignalObservation]s for this period per [SV]
/// ## Output
///    - formatted string when everything is correct
pub(crate) fn fmt_epoch(
    header: &Header,
    epoch: Epoch,
    flag: Epoch,
    clock_offset: &Option<f64>,
    signals: &[(SV, SignalObservation)],
) -> Result<String, Error> {
    if header.version.major < 3 {
        fmt_epoch_v2(header, key, clock_offset, signals)
    } else {
        fmt_epoch_v3(header, key, clock_offset, signals)
    }
}

/// Epoch (V3 record entry) formatter
/// ## Inputs
///    - header: Heavily [Header] dependent
///    - epoch: Ongoing [Epoch]
///    - flag: Ongoing [EpochFlag]
///    - clk_offset: possible Clock Offset [s]
///    - signals: [SignalObservation]s for this period per [SV]
/// ## Output
///    - formatted string when everything is correct
fn fmt_epoch_v3(
    header: &Header,
    epoch: Epoch,
    flag: Epoch,
    clock_offset: &Option<f64>,
    signals: &[(SV, SignalObservation)],
) -> Result<String, Error> {

    let mut lines = String::with_capacity(128);

    let total_sv = signals.iter().map(|(sv_j, _)| sv_j).unique().count();

    // retrieve system codes
    let observables = &header.obs.as_ref()
        .ok_or(Error::MissingObservableSpecs)?
        .codes;

    // format marker
    lines.push_str(&format!(
        "> {}  {} {:2}",
        format_epoch(epoch, Type::ObservationData, 3),
        flag,
        total_sv,
    ));

    // append ck offset when provided
    if let Some(data) = clock_offset {
        lines.push_str(&format!("{:13.4}", data));
    }

    lines.push('\n');

    for (sv, _) in signals.iter() {
        // line starts with SVNN
        lines.push_str(&format!("{:x}", sv));

        // retrive system codes
        let observables = match sv.constellation.is_sbas() {
            true => observables.get(&Constellation::SBAS),
            false => observables.get(&sv.constellation),
        };

        if observables.is_none() {
            continue ; // handles missing specs
        }

        let observables = observables.unwrap();

        // for each spec, try to obtain one measurement
        for observable in observables {
            if let Some((_, signal)) = signals.iter().filter(|(sv_j, sig_j)| {
                sv_j == sv && &sig_j.observable == observable
            })
            .reduce(|k, _| k) {

                // append measurement
                lines.push_str(&format!("{:14.3}", signal.value));

                if let Some(flag) = signal.lli {
                    lines.push_str(&format!("{}", flag.bits()));
                } else {
                    lines.push(' ');
                }

                if let Some(flag) = signal.snr {
                    lines.push_str(&format!("{:x}", flag));
                } else {
                    lines.push(' ');
                }
            } else {
                // missing measurement
                lines.push_str("                "); // TODO: improve
            }
        }
        lines.push('\n');
    }

    // improves rendition
    lines.truncate(lines.trim_end().len());

    Ok(lines)
}

/// Epoch (V<3 record entry) formatter.
/// Like any old system, V<3 is particularly painful to deal with.
/// ## Inputs
///    - header: Heavily [Header] dependent
///    - epoch: Ongoing [Epoch]
///    - flag: Ongoing [EpochFlag]
///    - clk_offset: possible Clock Offset [s]
///    - signals: [SignalObservation]s for this period per [SV]
/// ## Output
///    - formatted string when everything is correct
fn fmt_epoch_v2(
    header: &Header,
    epoch: Epoch,
    flag: Epoch,
    clock_offset: &Option<f64>,
    signals: &[(SV, SignalObservation)],
) -> Result<String, Error> {

    let mut lines = String::with_capacity(128);

    // retrieve system codes
    let observables = &header.obs
        .as_ref()
        .ok_or(Error::MissingObservableSpecs)?
        .codes;
    
    // format marker
    lines.push_str(&format!(
        " {}  {} {:2}",
        format_epoch(epoch, Type::ObservationData, 2),
        flag,
        data.len()
    ));

    let mut index = 0_u8;
    for (sv_index, (sv, _)) in data.iter().enumerate() {
        if index == 12 {
            index = 0;
            if sv_index == 12 {
                // first line
                if let Some(data) = clock_offset {
                    // push clock offsets
                    lines.push_str(&format!(" {:9.1}", data));
                }
            }
            lines.push_str("\n                                ");
        }
        lines.push_str(&format!("{:x}", sv));
        index += 1;
        /// ## Inputs
    }
    let obs_per_line = 5;
    // for each vehicle per epoch
    for (sv, observations) in data.iter() {
        // follow list of observables, as described in header section
        // for given constellation
        let observables = match sv.constellation.is_sbas() {
            true => observables.get(&Constellation::SBAS),
            false => observables.get(&sv.constellation),
        };
        if let Some(observables) = observables {
            for (obs_index, observable) in observables.iter().enumerate() {
                if obs_index % obs_per_line == 0 {
                    lines.push('\n');
                }
                if let Some(observation) = observations.get(observable) {
                    let formatted_obs = format!("{:14.3}", observation.obs);
                    let formatted_flags: String = match observation.lli {
                        Some(lli) => match observation.snr {
                            Some(snr) => format!("{}{:x}", lli.bits(), snr),
                            _ => format!("{} ", lli.bits()),
                        },
                        _ => match observation.snr {
                            Some(snr) => format!(" {:x}", snr),
                            _ => "  ".to_string(),
                        },
                    };
                    lines.push_str(&formatted_obs);
                    lines.push_str(&formatted_flags);
                } else {
                    // --> data is not provided: BLANK
                    lines.push_str("                ");
                }
            }
        }
    }
    lines
}

impl Merge for Record {
    /// Merge `rhs` into `Self`
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merge `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        for (rhs_epoch, (rhs_clk, rhs_vehicles)) in rhs {
            if let Some((clk, vehicles)) = self.get_mut(rhs_epoch) {
                // exact epoch (both timestamp and flag) did exist
                //  --> overwrite clock field (as is)
                *clk = *rhs_clk;
                // other fields:
                // either insert (if did not exist), or overwrite
                for (rhs_vehicle, rhs_observations) in rhs_vehicles {
                    if let Some(observations) = vehicles.get_mut(rhs_vehicle) {
                        for (rhs_observable, rhs_data) in rhs_observations {
                            if let Some(data) = observations.get_mut(rhs_observable) {
                                *data = *rhs_data; // overwrite
                            } else {
                                // new observation: insert it
                                observations.insert(rhs_observable.clone(), *rhs_data);
                            }
                        }
                    } else {
                        // new SV: insert it
                        vehicles.insert(*rhs_vehicle, rhs_observations.clone());
                    }
                }
            } else {
                // this epoch did not exist previously: insert it
                self.insert(*rhs_epoch, (*rhs_clk, rhs_vehicles.clone()));
            }
        }
        Ok(())
    }
}

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), SplitError> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if k.epoch < epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k.epoch >= epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((r0, r1))
    }
    fn split_dt(&self, duration: Duration) -> Result<Vec<Self>, SplitError> {
        let mut curr = Self::new();
        let mut ret: Vec<Self> = Vec::new();
        let mut prev: Option<Epoch> = None;
        for (key, data) in self {
            if let Some(p_epoch) = prev {
                let dt = *epoch - p_epoch;
                if dt >= duration {
                    prev = Some(*epoch);
                    ret.push(curr);
                    curr = Self::new();
                }
                curr.insert((*epoch, *flag), data.clone());
            } else {
                prev = Some(*epoch);
            }
        }
        Ok(ret)
    }
}

#[cfg(feature = "processing")]
fn repair_zero_mut(rec: &mut Record) {
    rec.retain(|_, meas| meas.value != 0.0);
}

#[cfg(feature = "processing")]
fn carrier_phase_cycles_mut(rec: &mut Record) {
    for (key, obs) in rec.iter_mut() {
        if key.observable.is_phase_observable() {
            if let Ok(carrier) = key.observable.carrier(sv.constellation) {
                obs.value *= carrier.wavelength();
            }
        }
    }
}

#[cfg(feature = "processing")]
fn null_phase_origin_mut(rec: &mut Record) {
    let mut t0 = HashMap::<(SV, Observable), f64>::new();
    for (key, obs) in rec.iter_mut() {
        if key.observable.is_phase_observable() {
            if let Some(t0) = t0.get(&(key.sv, key.observable.clone())) {
                obs.value -= t0;
            } else {
                t0.insert((key.sv, key.observable.clone()), obs.value);
                obs.value = 0.0_f64;
            }
        }
    }
}

#[cfg(feature = "processing")]
pub(crate) fn repair_mut(rec: &mut Record, repair: Repair) {
    match repair {
        Repair::Zero => repair_zero_mut(rec),
        Repair::NullPhaseOrigin => null_phase_origin_mut(rec),
        Repair::CarrierPhaseCycles => carrier_phase_cycles_mut(rec),
    }
}

#[cfg(feature = "processing")]
pub(crate) fn observation_mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e == *epoch),
            FilterItem::ClockItem => {
                rec.retain(|_, (clk, _)| clk.is_some());
            },
            FilterItem::ConstellationItem(constells) => {
                let mut broad_sbas_filter = false;
                for c in constells {
                    broad_sbas_filter |= *c == Constellation::SBAS;
                }
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| {
                        if broad_sbas_filter {
                            sv.constellation.is_sbas() || constells.contains(&sv.constellation)
                        } else {
                            constells.contains(&sv.constellation)
                        }
                    });
                    !svs.is_empty()
                });
            },
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| items.contains(sv));
                    !svs.is_empty()
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, (_, svs)| {
                    svs.retain(|_, obs| {
                        obs.retain(|_, data| {
                            if let Some(snr) = data.snr {
                                snr == filter
                            } else {
                                false // no snr: drop out
                            }
                        });
                        !obs.is_empty()
                    });
                    !svs.is_empty()
                });
            },
            FilterItem::ComplexItem(filter) => {
                // try to interprate as [Observable]
                let observables = filter
                    .iter()
                    .filter_map(|f| {
                        if let Ok(ob) = Observable::from_str(f) {
                            Some(ob)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if observables.len() > 0 {
                    rec.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|ob, _| observables.contains(&ob));
                            !obs.is_empty()
                        });
                        !svs.is_empty()
                    });
                }
            },
            _ => {},
        },
        MaskOperand::NotEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e != *epoch),
            FilterItem::ClockItem => {
                rec.retain(|_, (clk, _)| clk.is_none());
            },
            FilterItem::ConstellationItem(constells) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| !constells.contains(&sv.constellation));
                    !svs.is_empty()
                });
            },
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| !items.contains(sv));
                    !svs.is_empty()
                });
            },
            FilterItem::ComplexItem(filter) => {
                // try to interprate as [Observable]
                let observables = filter
                    .iter()
                    .filter_map(|f| {
                        if let Ok(ob) = Observable::from_str(f) {
                            Some(ob)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if observables.len() > 0 {
                    rec.retain(|_, (_, svs)| {
                        svs.retain(|_, obs| {
                            obs.retain(|ob, _| !observables.contains(&ob));
                            !obs.is_empty()
                        });
                        !svs.is_empty()
                    });
                }
            },
            _ => {},
        },
        MaskOperand::GreaterEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e >= *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| {
                        let mut retain = true;
                        for item in items {
                            if item.constellation == sv.constellation {
                                retain = sv.prn >= item.prn;
                            }
                        }
                        retain
                    });
                    !svs.is_empty()
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, (_, svs)| {
                    svs.retain(|_, obs| {
                        obs.retain(|_, data| {
                            if let Some(snr) = data.snr {
                                snr >= filter
                            } else {
                                false // no snr: drop out
                            }
                        });
                        !obs.is_empty()
                    });
                    !svs.is_empty()
                });
            },
            _ => {},
        },
        MaskOperand::GreaterThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e > *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| {
                        let mut retain = true;
                        for item in items {
                            if item.constellation == sv.constellation {
                                retain = sv.prn > item.prn;
                            }
                        }
                        retain
                    });
                    !svs.is_empty()
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, (_, svs)| {
                    svs.retain(|_, obs| {
                        obs.retain(|_, data| {
                            if let Some(snr) = data.snr {
                                snr > filter
                            } else {
                                false // no snr: drop out
                            }
                        });
                        !obs.is_empty()
                    });
                    !svs.is_empty()
                });
            },
            _ => {},
        },
        MaskOperand::LowerEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e <= *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| {
                        let mut retain = true;
                        for item in items {
                            if item.constellation == sv.constellation {
                                retain = sv.prn <= item.prn;
                            }
                        }
                        retain
                    });
                    !svs.is_empty()
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, (_, svs)| {
                    svs.retain(|_, obs| {
                        obs.retain(|_, data| {
                            if let Some(snr) = data.snr {
                                snr <= filter
                            } else {
                                false // no snr: drop out
                            }
                        });
                        !obs.is_empty()
                    });
                    !svs.is_empty()
                });
            },
            _ => {},
        },
        MaskOperand::LowerThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|(e, _), _| *e < *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, (_, svs)| {
                    svs.retain(|sv, _| {
                        let mut retain = true;
                        for item in items {
                            if item.constellation == sv.constellation {
                                retain = sv.prn < item.prn;
                            }
                        }
                        retain
                    });
                    !svs.is_empty()
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, (_, svs)| {
                    svs.retain(|_, obs| {
                        obs.retain(|_, data| {
                            if let Some(snr) = data.snr {
                                snr < filter
                            } else {
                                false // no snr: drop out
                            }
                        });
                        !obs.is_empty()
                    });
                    !svs.is_empty()
                });
            },
            _ => {},
        },
    }
}

#[cfg(feature = "processing")]
pub(crate) fn observation_decim_mut(rec: &mut Record, decim: &DecimationFilter) {
    if decim.item.is_some() {
        todo!("targetted decimation not supported yet");
    }
    match decim.filter {
        DecimationFilterType::Modulo(r) => {
            let mut i = 0;
            rec.retain(|_, _| {
                let retained = (i % r) == 0;
                i += 1;
                retained
            });
        },
        DecimationFilterType::Duration(interval) => {
            let mut last_retained = Option::<Epoch>::None;
            rec.retain(|(e, _), _| {
                if let Some(last) = last_retained {
                    let dt = *e - last;
                    if dt >= interval {
                        last_retained = Some(*e);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(*e);
                    true // always retain 1st epoch
                }
            });
        },
    }
}

#[cfg(feature = "obs")]
use crate::observation::{Combination, Combine};

/*
 * Combines same physics but observed on different carrier frequency
 */
#[cfg(feature = "obs")]
fn dual_freq_combination(
    rec: &Record,
    combination: Combination,
) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
    let mut ret: HashMap<
        (Observable, Observable),
        BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>,
    > = HashMap::new();
    for (epoch, (_, vehicles)) in rec {
        for (sv, observations) in vehicles {
            for (lhs_observable, lhs_data) in observations {
                if !lhs_observable.is_phase_observable()
                    && !lhs_observable.is_pseudorange_observable()
                {
                    continue; // only for these two physics
                }

                // consider anything but L1
                let lhs_code = lhs_observable.to_string();
                let lhs_is_l1 = lhs_code.contains('1');
                if lhs_is_l1 {
                    continue;
                }

                // find L1 reference observation
                let mut reference: Option<(Observable, f64)> = None;
                for (ref_observable, ref_data) in observations {
                    let mut shared_physics = ref_observable.is_phase_observable()
                        && lhs_observable.is_phase_observable();
                    shared_physics |= ref_observable.is_pseudorange_observable()
                        && lhs_observable.is_pseudorange_observable();
                    if !shared_physics {
                        continue;
                    }

                    let refcode = ref_observable.to_string();
                    if refcode.contains('1') {
                        reference = Some((ref_observable.clone(), ref_data.obs));
                        break; // DONE searching
                    }
                }

                if reference.is_none() {
                    continue; // can't proceed further
                }
                let (ref_observable, ref_data) = reference.unwrap();

                // determine frequencies
                let lhs_carrier = Carrier::from_observable(sv.constellation, lhs_observable);
                let ref_carrier = Carrier::from_observable(sv.constellation, &ref_observable);
                if lhs_carrier.is_err() | ref_carrier.is_err() {
                    continue; // undetermined frequency
                }

                let (lhs_carrier, ref_carrier) = (lhs_carrier.unwrap(), ref_carrier.unwrap());
                let (fj, fi) = (lhs_carrier.frequency(), ref_carrier.frequency());
                let (lambda_j, lambda_i) = (lhs_carrier.wavelength(), ref_carrier.wavelength());

                let alpha = match combination {
                    Combination::GeometryFree => 1.0_f64,
                    Combination::IonosphereFree => 1.0 / (fi.powi(2) - fj.powi(2)),
                    Combination::WideLane => 1.0 / (fi - fj),
                    Combination::NarrowLane => 1.0 / (fi + fj),
                    Combination::MelbourneWubbena => unreachable!("mw combination"),
                };

                let beta = match combination {
                    Combination::GeometryFree => 1.0_f64,
                    Combination::IonosphereFree => fi.powi(2),
                    Combination::WideLane | Combination::NarrowLane => fi,
                    Combination::MelbourneWubbena => unreachable!("mw combination"),
                };

                let gamma = match combination {
                    Combination::GeometryFree => 1.0_f64,
                    Combination::IonosphereFree => fj.powi(2),
                    Combination::WideLane | Combination::NarrowLane => fj,
                    Combination::MelbourneWubbena => unreachable!("mw combination"),
                };

                let (v_j, v_i) = match combination {
                    Combination::GeometryFree => {
                        if ref_observable.is_pseudorange_observable() {
                            (ref_data, lhs_data.obs)
                        } else {
                            (lhs_data.obs * lambda_j, ref_data * lambda_i)
                        }
                    },
                    _ => {
                        if ref_observable.is_pseudorange_observable() {
                            (lhs_data.obs, ref_data)
                        } else {
                            (lhs_data.obs * lambda_j, ref_data * lambda_i)
                        }
                    },
                };

                let value = match combination {
                    Combination::NarrowLane => alpha * (beta * v_i + gamma * v_j),
                    _ => alpha * (beta * v_i - gamma * v_j),
                };

                let combination = (lhs_observable.clone(), ref_observable.clone());
                if let Some(data) = ret.get_mut(&combination) {
                    if let Some(data) = data.get_mut(sv) {
                        data.insert(*epoch, value);
                    } else {
                        let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                        map.insert(*epoch, value);
                        data.insert(*sv, map);
                    }
                } else {
                    let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                    map.insert(*epoch, value);
                    let mut bmap: BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>> = BTreeMap::new();
                    bmap.insert(*sv, map);
                    ret.insert(combination, bmap);
                }
            }
        }
    }
    ret
}

#[cfg(feature = "obs")]
fn mw_combination(
    rec: &Record,
) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
    let code_narrow = dual_freq_combination(rec, Combination::NarrowLane);
    let mut phase_wide = dual_freq_combination(rec, Combination::WideLane);

    phase_wide.retain(|(lhs_obs, rhs_obs), phase_wide| {
        let lhs_code_obs =
            Observable::from_str(&format!("C{}", &lhs_obs.to_string()[1..])).unwrap();
        let rhs_code_obs =
            Observable::from_str(&format!("C{}", &rhs_obs.to_string()[1..])).unwrap();

        if lhs_obs.is_phase_observable() {
            if let Some(code_data) = code_narrow.get(&(lhs_code_obs, rhs_code_obs)) {
                phase_wide.retain(|sv, phase_data| {
                    if let Some(code_data) = code_data.get(sv) {
                        phase_data.retain(|epoch, _| code_data.get(epoch).is_some());
                        !phase_data.is_empty()
                    } else {
                        false
                    }
                });
                !phase_wide.is_empty()
            } else {
                false
            }
        } else {
            false
        }
    });

    for ((lhs_obs, rhs_obs), phase_data) in phase_wide.iter_mut() {
        let lhs_code_obs =
            Observable::from_str(&format!("C{}", &lhs_obs.to_string()[1..])).unwrap();
        let rhs_code_obs =
            Observable::from_str(&format!("C{}", &rhs_obs.to_string()[1..])).unwrap();

        if let Some(code_data) = code_narrow.get(&(lhs_code_obs, rhs_code_obs)) {
            for (phase_sv, data) in phase_data {
                if let Some(code_data) = code_data.get(phase_sv) {
                    for (epoch, phase_wide) in data {
                        if let Some(narrow_code) = code_data.get(epoch) {
                            *phase_wide -= narrow_code;
                        }
                    }
                }
            }
        }
    }
    phase_wide
}

#[cfg(feature = "obs")]
impl Combine for Record {
    fn combine(
        &self,
        c: Combination,
    ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
        match c {
            Combination::GeometryFree
            | Combination::IonosphereFree
            | Combination::NarrowLane
            | Combination::WideLane => dual_freq_combination(self, c),
            Combination::MelbourneWubbena => mw_combination(self),
        }
    }
}

#[cfg(feature = "obs")]
use crate::{
    carrier,
    observation::Dcb, //Mp},
};

#[cfg(feature = "obs")]
impl Dcb for Record {
    fn dcb(&self) -> HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
        let mut ret: HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> =
            HashMap::new();
        for (epoch, (_, vehicles)) in self {
            for (sv, observations) in vehicles {
                for (lhs_observable, lhs_observation) in observations {
                    if !lhs_observable.is_phase_observable()
                        && !lhs_observable.is_pseudorange_observable()
                    {
                        continue;
                    }
                    let lhs_code = lhs_observable.to_string();
                    let lhs_carrier = &lhs_code[1..2];
                    let lhs_code = &lhs_code[1..];

                    for rhs_code in carrier::KNOWN_CODES.iter() {
                        // locate a reference code
                        if *rhs_code != lhs_code {
                            // code differs
                            if rhs_code.starts_with(lhs_carrier) {
                                // same carrier
                                let tolocate = match lhs_observable.is_phase_observable() {
                                    true => "L".to_owned() + rhs_code,  // same physics
                                    false => "C".to_owned() + rhs_code, // same physics
                                };
                                let tolocate = Observable::from_str(&tolocate).unwrap();
                                if let Some(rhs_observation) = observations.get(&tolocate) {
                                    // got a reference code
                                    let mut already_diffd = false;

                                    for (op, vehicles) in ret.iter_mut() {
                                        if op.contains(lhs_code) {
                                            already_diffd = true;

                                            // determine this code's role in the diff op
                                            // so it remains consistent
                                            let items: Vec<&str> = op.split('-').collect();

                                            if lhs_code == items[0] {
                                                // code is differenced
                                                if let Some(data) = vehicles.get_mut(sv) {
                                                    data.insert(
                                                        *epoch,
                                                        lhs_observation.obs - rhs_observation.obs,
                                                    );
                                                } else {
                                                    let mut bmap: BTreeMap<
                                                        (Epoch, EpochFlag),
                                                        f64,
                                                    > = BTreeMap::new();
                                                    bmap.insert(
                                                        *epoch,
                                                        lhs_observation.obs - rhs_observation.obs,
                                                    );
                                                    vehicles.insert(*sv, bmap);
                                                }
                                            } else {
                                                // code is refered to
                                                if let Some(data) = vehicles.get_mut(sv) {
                                                    data.insert(
                                                        *epoch,
                                                        rhs_observation.obs - lhs_observation.obs,
                                                    );
                                                } else {
                                                    let mut bmap: BTreeMap<
                                                        (Epoch, EpochFlag),
                                                        f64,
                                                    > = BTreeMap::new();
                                                    bmap.insert(
                                                        *epoch,
                                                        rhs_observation.obs - lhs_observation.obs,
                                                    );
                                                    vehicles.insert(*sv, bmap);
                                                }
                                            }
                                        }
                                    }
                                    if !already_diffd {
                                        let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
                                            BTreeMap::new();
                                        bmap.insert(
                                            *epoch,
                                            lhs_observation.obs - rhs_observation.obs,
                                        );
                                        let mut map: BTreeMap<
                                            SV,
                                            BTreeMap<(Epoch, EpochFlag), f64>,
                                        > = BTreeMap::new();
                                        map.insert(*sv, bmap);
                                        ret.insert(format!("{}-{}", lhs_code, rhs_code), map);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        ret
    }
}

#[cfg(feature = "obs")]
pub(crate) fn lli_and_mask_mut(rec: &mut Record, mask: LliFlags) {
    rec.retain(|_, obs| {
        if let Some(lli) = obs.lli {
            lli.intersects(mask)
        } else {
            false
        }
    })
}

#[cfg(feature = "obs")]
pub(crate) fn phase_lock_loss_mut(rec: &mut Record) {
    rec.retain(|key, obs| {
        if key.observable.is_phase_observable() {
            if let Some(lli) = obs.lli {
                !lli.intersects(LliFlags::LOCK_LOSS)
            } else {
                true
            }
        } else {
            true
        }
    });
}

/*
 * Code multipath bias
 */
#[cfg(feature = "obs")]
pub(crate) fn code_multipath(rec: &Record) -> HashMap<ObsKey, Observation> {
    Default::default()
}
//    let mut ret: HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> =
//        HashMap::new();
//
//    for (epoch, (_, vehicles)) in rec {
//        for (sv, observations) in vehicles {
//            for (observable, obsdata) in observations {
//                if !observable.is_pseudorange_observable() {
//                    continue;
//                }
//
//                let code = observable.to_string();
//                let carrier = &code[1..2].to_string();
//                let code_is_l1 = code.contains('1');
//
//                let mut phase_i = Option::<f64>::None;
//                let mut phase_j = Option::<f64>::None;
//                let mut f_i = Option::<f64>::None;
//                let mut f_j = Option::<f64>::None;
//
//                for (rhs_observable, rhs_data) in observations {
//                    if !rhs_observable.is_phase_observable() {
//                        continue;
//                    }
//                    let rhs_code = rhs_observable.to_string();
//
//                    // identify carrier signal
//                    let rhs_carrier = Carrier::from_observable(sv.constellation, rhs_observable);
//                    if rhs_carrier.is_err() {
//                        continue;
//                    }
//                    let rhs_carrier = rhs_carrier.unwrap();
//                    let lambda = rhs_carrier.wavelength();
//
//                    if code_is_l1 {
//                        if rhs_code.contains('2') {
//                            f_j = Some(rhs_carrier.frequency());
//                            phase_j = Some(rhs_data.obs * lambda);
//                        } else if rhs_code.contains(carrier) {
//                            f_i = Some(rhs_carrier.frequency());
//                            phase_i = Some(rhs_data.obs * lambda);
//                        }
//                    } else if rhs_code.contains('1') {
//                        f_j = Some(rhs_carrier.frequency());
//                        phase_j = Some(rhs_data.obs * lambda);
//                    } else if rhs_code.contains(carrier) {
//                        f_i = Some(rhs_carrier.frequency());
//                        phase_i = Some(rhs_data.obs * lambda);
//                    }
//
//                    if phase_i.is_some() && phase_j.is_some() {
//                        break; // DONE
//                    }
//                }
//
//                if phase_i.is_none() || phase_j.is_none() {
//                    continue; // can't proceed
//                }
//
//                let gamma = (f_i.unwrap() / f_j.unwrap()).powi(2);
//                let alpha = (gamma + 1.0) / (gamma - 1.0);
//                let beta = 2.0 / (gamma - 1.0);
//                let value = obsdata.obs - alpha * phase_i.unwrap() + beta * phase_j.unwrap();
//
//                if let Some(data) = ret.get_mut(observable) {
//                    if let Some(data) = data.get_mut(sv) {
//                        data.insert(*epoch, value);
//                    } else {
//                        let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
//                        map.insert(*epoch, value);
//                        data.insert(*sv, map);
//                    }
//                } else {
//                    let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
//                    map.insert(*epoch, value);
//                    let mut bmap: BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>> = BTreeMap::new();
//                    bmap.insert(*sv, map);
//                    ret.insert(observable.clone(), bmap);
//                }
//            }
//        }
//    }
//    ret
//}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        header::Header,
        version::Version,
        observation::Observation,
        epoch::parse_utc as parse_utc_epoch,
        prelude::{TimeScale, SV},
    };

    fn parse_and_format_helper(ver: Version, epoch_str: &str, expected_flag: EpochFlag) {
        let first = parse_utc_epoch("2020 01 01 00 00  0.1000000").unwrap();
        let data: BTreeMap<SV, HashMap<Observable, Observation>> = BTreeMap::new();
        let header = Header::default()
            .with_version(ver)
            .with_observation_fields(HeaderFields::default().with_time_of_first_obs(first));
        let ts = TimeScale::UTC;
        let clock_offset: Option<f64> = None;

        let e = parse_epoch(&header, epoch_str, ts);

        match expected_flag {
            EpochFlag::Ok | EpochFlag::PowerFailure | EpochFlag::CycleSlip => {
                assert!(e.is_ok())
            },
            _ => {
                // TODO: Update alongside parse_event
                assert!(e.is_err());
                return;
            },
        }
        let ((e, flag), _, _) = e.unwrap();
        assert_eq!(flag, expected_flag);
        if ver.major < 3 {
            assert_eq!(
                fmt_epoch_v2(e, flag, &clock_offset, &data, &header)
                    .lines()
                    .next()
                    .unwrap(),
                epoch_str
            );
        } else {
            assert_eq!(
                fmt_epoch_v3(e, flag, &clock_offset, &data, &header)
                    .lines()
                    .next()
                    .unwrap(),
                epoch_str
            );
        }
    }

    #[test]
    fn obs_v2_parse_and_format() {
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  0  0",
            EpochFlag::Ok,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  1  0",
            EpochFlag::PowerFailure,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  2  0",
            EpochFlag::AntennaBeingMoved,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  3  0",
            EpochFlag::NewSiteOccupation,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  4  0",
            EpochFlag::HeaderInformationFollows,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  5  0",
            EpochFlag::ExternalEvent,
        );
        parse_and_format_helper(
            Version { major: 2, minor: 0 },
            " 21 12 21  0  0 30.0000000  6  0",
            EpochFlag::CycleSlip,
        );
    }
    #[test]
    fn obs_v3_parse_and_format() {
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  0  0",
            EpochFlag::Ok,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  1  0",
            EpochFlag::PowerFailure,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  2  0",
            EpochFlag::AntennaBeingMoved,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  3  0",
            EpochFlag::NewSiteOccupation,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  4  0",
            EpochFlag::HeaderInformationFollows,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  5  0",
            EpochFlag::ExternalEvent,
        );
        parse_and_format_helper(
            Version { major: 3, minor: 0 },
            "> 2021 12 21 00 00 30.0000000  6  0",
            EpochFlag::CycleSlip,
        );
    }
}
