//! PPP solver
use crate::{
    cli::Context,
    positioning::{
        bd_model, cast_rtk_carrier, kb_model, ng_model, ClockStateProvider, EphemerisSource,
    },
};

use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
};

use rinex::{
    carrier::Carrier,
    observation::LliFlags,
    prelude::{Observable, SV},
};

mod report;
pub use report::Report;

pub mod post_process;

use gnss_rtk::prelude::{
    Candidate, Epoch, IonoComponents, Observation, OrbitSource, PVTSolution, Solver,
    TropoComponents,
};

pub fn resolve<'a, 'b, CK: ClockStateProvider, O: OrbitSource>(
    ctx: &Context,
    eph: &'a RefCell<EphemerisSource<'b>>,
    mut clock: CK,
    mut solver: Solver<O>,
    // rx_lat_ddeg: f64,
) -> BTreeMap<Epoch, PVTSolution> {
    let mut past_epoch = Option::<Epoch>::None;

    let mut solutions: BTreeMap<Epoch, PVTSolution> = BTreeMap::new();

    // infaillible, at this point
    let obs_data = ctx.data.observation().unwrap();

    let mut candidates = Vec::<Candidate>::with_capacity(4);
    let mut sv_observations = HashMap::<SV, Vec<Observation>>::new();

    // TODO: RTK
    let mut remote_observations = Vec::<Observation>::new();

    for (t, signal) in obs_data.signal_observations_sampling_ok_iter() {
        let carrier = Carrier::from_observable(signal.sv.constellation, &signal.observable);
        if carrier.is_err() {
            continue;
        }

        let carrier = carrier.unwrap();
        let rtk_carrier = cast_rtk_carrier(carrier);

        if let Some(past_t) = past_epoch {
            if t != past_t {
                // New epoch: solving attempt
                for (sv, observations) in sv_observations.iter() {
                    // Create new candidate
                    let mut cd = Candidate::new(*sv, past_t, observations.clone());

                    // candidate "fixup" or customizations
                    match clock.next_clock_at(past_t, *sv) {
                        Some(dt) => cd.set_clock_correction(dt),
                        None => error!("{} ({}) - no clock correction available", past_t, *sv),
                    }

                    if let Some((_, _, eph)) = eph.borrow_mut().select(past_t, *sv) {
                        if let Some(tgd) = eph.tgd() {
                            debug!("{} ({}) - tgd: {}", past_t, *sv, tgd);
                            cd.set_group_delay(tgd);
                        }
                    }

                    let tropo = TropoComponents::Unknown;
                    cd.set_tropo_components(tropo);

                    let mut iono = IonoComponents::Unknown;

                    match ctx.data.brdc_navigation() {
                        Some(brdc) => {
                            if let Some(model) = kb_model(brdc, past_t) {
                                iono = IonoComponents::KbModel(model);
                            } else if let Some(model) = ng_model(brdc, past_t) {
                                iono = IonoComponents::NgModel(model);
                            } else if let Some(model) = bd_model(brdc, past_t) {
                                iono = IonoComponents::BdModel(model);
                            }
                        },
                        None => {
                            cd.set_iono_components(IonoComponents::Unknown);
                        },
                    }

                    match iono {
                        IonoComponents::Unknown => {
                            warn!("{} ({}) - undefined ionosphere parameters", past_t, *sv)
                        },
                        IonoComponents::KbModel(_) => info!(
                            "{} ({}) - using KLOBUCHAR ionosphere parameters",
                            past_t, *sv
                        ),
                        IonoComponents::NgModel(_) => info!(
                            "{} ({}) - using NEQUICK-G ionosphere parameters",
                            past_t, *sv
                        ),
                        IonoComponents::BdModel(_) => {
                            info!("{} ({}) - using BDGIM ionosphere parameters", past_t, *sv)
                        },
                        _ => {},
                    }

                    cd.set_iono_components(iono);

                    candidates.push(cd);
                }

                match solver.resolve(t, &candidates) {
                    Ok((t, pvt)) => {
                        solutions.insert(t, pvt);
                    },
                    Err(e) => warn!("{} : pvt solver error \"{}\"", t, e),
                }

                candidates.clear();
                sv_observations.clear();
                remote_observations.clear();
            }
        }

        if let Some((_, observations)) = sv_observations
            .iter_mut()
            .filter(|(k, _)| **k == signal.sv)
            .reduce(|k, _| k)
        {
            if let Some(observation) = observations
                .iter_mut()
                .filter(|k| k.carrier == rtk_carrier)
                .reduce(|k, _| k)
            {
                match signal.observable {
                    Observable::PhaseRange(_) => {
                        observation.set_ambiguous_phase_range(signal.value);
                    },
                    Observable::PseudoRange(_) => {
                        observation.set_pseudo_range(signal.value);
                    },
                    Observable::Doppler(_) => {
                        observation.set_doppler(signal.value);
                    },
                    _ => {},
                }
            } else {
                match signal.observable {
                    Observable::PhaseRange(_) => {
                        observations.push(Observation::ambiguous_phase_range(
                            rtk_carrier,
                            signal.value,
                            None,
                        ));
                    },
                    Observable::PseudoRange(_) => {
                        observations.push(Observation::pseudo_range(
                            rtk_carrier,
                            signal.value,
                            None,
                        ));
                    },
                    Observable::Doppler(_) => {
                        observations.push(Observation::doppler(rtk_carrier, signal.value, None));
                    },
                    _ => {},
                }
            }
        } else {
            match signal.observable {
                Observable::PhaseRange(_) => {
                    sv_observations.insert(
                        signal.sv,
                        vec![Observation::ambiguous_phase_range(
                            rtk_carrier,
                            signal.value,
                            None,
                        )],
                    );
                },
                Observable::PseudoRange(_) => {
                    sv_observations.insert(
                        signal.sv,
                        vec![Observation::pseudo_range(rtk_carrier, signal.value, None)],
                    );
                },
                Observable::Doppler(_) => {
                    sv_observations.insert(
                        signal.sv,
                        vec![Observation::doppler(rtk_carrier, signal.value, None)],
                    );
                },
                _ => {},
            }
        }

        past_epoch = Some(t);
    }
    solutions
}
