//! PPP solver
use crate::{
    cli::Context,
    positioning::{
        bd_model, cast_rtk_carrier, kb_model, ng_model, ClockStateProvider, EphemerisSource,
        RemoteRTKReference,
    },
};

use itertools::Itertools;

use std::{cell::RefCell, collections::BTreeMap};

use rinex::prelude::Observable;

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
    base_station: &'a mut RemoteRTKReference,
    mut solver: Solver<O>,
    // rx_lat_ddeg: f64,
) -> BTreeMap<Epoch, PVTSolution> {
    // returned solutions
    let mut solutions: BTreeMap<Epoch, PVTSolution> = BTreeMap::new();

    // infaillible at this point
    let obs_data = ctx.data.observation().unwrap();

    // TODO: Optional remote reference site
    // let rtk_compatible = ctx.rtk_compatible();
    // let remote_site = ctx.reference_site.as_ref();

    for (key, observations) in obs_data.observations_iter() {
        // TODO: this core logic should take the LLI
        // flags into account, for every single phase data point as well.
        // Currently only the sampling (Epoch Flag) is taken into account.
        // This will garantee correct PPP processing.
        if !key.flag.is_ok() {
            continue;
        }

        let t = key.epoch;

        // try to gather remote observation
        //if let Some(remote) = base_station.observe(*t, *sv, carrier) {
        //    remote_observations.push(remote);
        //}
        let mut candidates = Vec::<Candidate>::with_capacity(4);

        for sv in observations.signals.iter().map(|sig| sig.sv).unique() {
            let observations = observations
                .signals
                .iter()
                .filter_map(|sig| {
                    if sig.sv == sv {
                        if let Ok(carrier) = sig.observable.carrier(sig.sv.constellation) {
                            let carrier = cast_rtk_carrier(carrier);

                            match &sig.observable {
                                Observable::PseudoRange(_) => Some(Observation::pseudo_range(
                                    carrier,
                                    sig.value,
                                    sig.snr.map(|snr| snr.into()),
                                )),
                                Observable::PhaseRange(_) => {
                                    Some(Observation::ambiguous_phase_range(
                                        carrier,
                                        sig.value,
                                        sig.snr.map(|snr| snr.into()),
                                    ))
                                },
                                Observable::Doppler(_) => Some(Observation::doppler(
                                    carrier,
                                    sig.value,
                                    sig.snr.map(|snr| snr.into()),
                                )),
                                //Observable::SSI(code) => {
                                //},
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            // create [Candidate]
            let mut candidate = Candidate::new(sv, t, observations.clone());

            // customization: clock corr
            match clock.next_clock_at(t, sv) {
                Some(dt) => {
                    candidate.set_clock_correction(dt);
                },
                None => {
                    error!("{} ({}) - no clock correction available", t, sv);
                },
            }

            // customization: TGD
            if let Some((_, _, eph)) = eph.borrow_mut().select(t, sv) {
                if let Some(tgd) = eph.tgd() {
                    debug!("{} ({}) - tgd: {}", t, sv, tgd);
                    candidate.set_group_delay(tgd);
                }
            }

            // customization: Tropo (TODO: METEO)
            let tropo = TropoComponents::Unknown;
            candidate.set_tropo_components(tropo);

            // customization: Iono
            match ctx.data.brdc_navigation() {
                Some(brdc) => {
                    if let Some(model) = kb_model(brdc, t) {
                        candidate.set_iono_components(IonoComponents::KbModel(model));
                    } else if let Some(model) = ng_model(brdc, t) {
                        candidate.set_iono_components(IonoComponents::NgModel(model));
                    } else if let Some(model) = bd_model(brdc, t) {
                        candidate.set_iono_components(IonoComponents::BdModel(model));
                    } else {
                        //TODO STEC/IONEX
                        candidate.set_iono_components(IonoComponents::Unknown);
                    }
                },
                None => {
                    candidate.set_iono_components(IonoComponents::Unknown);
                },
            }

            // Customization: Remote
            //if !remote_observations.is_empty() {
            //    candidate.set_remote_observations(remote_observations);
            //}

            candidates.push(candidate);
        }

        match solver.resolve(t, &candidates) {
            Ok((t, pvt)) => {
                solutions.insert(t, pvt);
            },
            Err(err) => warn!("{} : pvt solver error \"{}\"", key.epoch, err),
        }
    }

    solutions
}
