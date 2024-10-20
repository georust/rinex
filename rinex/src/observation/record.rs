//! OBS RINEX specific content

// TODO: can't we get rid of num_integer ?
use num_integer::div_ceil;

#[cfg(feature = "log")]
use log::{debug, error};

#[cfg(feature = "processing")]
use qc_traits::processing::{
    DecimationFilter, DecimationFilterType, FilterItem, MaskFilter, MaskOperand, Repair,
};

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

/*
 * Code multipath bias
 */
#[cfg(feature = "obs")]
pub(crate) fn code_multipath(
    rec: &Record,
) -> HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
    let mut ret: HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> =
        HashMap::new();

    for (epoch, (_, vehicles)) in rec {
        for (sv, observations) in vehicles {
            for (observable, obsdata) in observations {
                if !observable.is_pseudorange_observable() {
                    continue;
                }

                let code = observable.to_string();
                let carrier = &code[1..2].to_string();
                let code_is_l1 = code.contains('1');

                let mut phase_i = Option::<f64>::None;
                let mut phase_j = Option::<f64>::None;
                let mut f_i = Option::<f64>::None;
                let mut f_j = Option::<f64>::None;

                for (rhs_observable, rhs_data) in observations {
                    if !rhs_observable.is_phase_observable() {
                        continue;
                    }
                    let rhs_code = rhs_observable.to_string();

                    // identify carrier signal
                    let rhs_carrier = Carrier::from_observable(sv.constellation, rhs_observable);
                    if rhs_carrier.is_err() {
                        continue;
                    }
                    let rhs_carrier = rhs_carrier.unwrap();
                    let lambda = rhs_carrier.wavelength();

                    if code_is_l1 {
                        if rhs_code.contains('2') {
                            f_j = Some(rhs_carrier.frequency());
                            phase_j = Some(rhs_data.obs * lambda);
                        } else if rhs_code.contains(carrier) {
                            f_i = Some(rhs_carrier.frequency());
                            phase_i = Some(rhs_data.obs * lambda);
                        }
                    } else if rhs_code.contains('1') {
                        f_j = Some(rhs_carrier.frequency());
                        phase_j = Some(rhs_data.obs * lambda);
                    } else if rhs_code.contains(carrier) {
                        f_i = Some(rhs_carrier.frequency());
                        phase_i = Some(rhs_data.obs * lambda);
                    }

                    if phase_i.is_some() && phase_j.is_some() {
                        break; // DONE
                    }
                }

                if phase_i.is_none() || phase_j.is_none() {
                    continue; // can't proceed
                }

                let gamma = (f_i.unwrap() / f_j.unwrap()).powi(2);
                let alpha = (gamma + 1.0) / (gamma - 1.0);
                let beta = 2.0 / (gamma - 1.0);
                let value = obsdata.obs - alpha * phase_i.unwrap() + beta * phase_j.unwrap();

                if let Some(data) = ret.get_mut(observable) {
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
                    ret.insert(observable.clone(), bmap);
                }
            }
        }
    }
    ret
}

#[cfg(test)]
mod test {
    use super::*;
    fn parse_and_format_helper(ver: Version, epoch_str: &str, expected_flag: EpochFlag) {
        let first = parse_utc_epoch("2020 01 01 00 00  0.1000000").unwrap();
        let data: BTreeMap<SV, HashMap<Observable, ObservationData>> = BTreeMap::new();
        let header = Header::default().with_version(ver).with_observation_fields(
            crate::observation::HeaderFields::default().with_time_of_first_obs(first),
        );
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
