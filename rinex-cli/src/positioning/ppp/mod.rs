//! PPP solver
use crate::{
    cli::Context,
    positioning::{
        bd_model, cast_rtk_carrier, kb_model, ng_model, ClockStateProvider, EphemerisSource,
        RemoteRTKReference,
    },
};

use std::{cell::RefCell, collections::BTreeMap};

use rinex::{
    carrier::Carrier, observation::LliFlags,
    prelude::Observable,
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
    base_station: &'a mut RemoteRTKReference,
    mut solver: Solver<O>,
    // rx_lat_ddeg: f64,
) -> BTreeMap<Epoch, PVTSolution> {

    let mut past_epoch = Option::<Epoch>::None;

    let mut solutions: BTreeMap<Epoch, PVTSolution> = BTreeMap::new();

    // infaillible, at this point
    let obs_data = ctx.data.observation().unwrap();

    // Optional remote reference site
    // let rtk_compatible = ctx.rtk_compatible();
    // let remote_site = ctx.reference_site.as_ref();

    let mut candidates = Vec::<Candidate>::with_capacity(4);
    let mut observations = Vec::<Observation>::new();
    let mut remote_observations = Vec::<Observation>::new();

    for (t, signal) in obs_data.signal_observations_sampling_ok_iter() {
        
        match signal.observable {
            Observable::PhaseRange(_) => {
            },
            Observable::Doppler(_) => {

            },
            Observable::PseudoRange(_) => {

            },
            _ => {
                continue; // not interesting
            }
        }

        if let Some(lli) = signal.lli {
            if lli != LliFlags::OK_OR_UNKNOWN {
                // TODO : manage this event
                warn!("{}({}) - {:?}", t, signal.sv, lli);
            }    
        }

        let carrier = Carrier::from_observable(signal.sv.constellation, signal.observable);
        if carrier.is_err() {
            continue ;
        }

        let carrier = carrier.unwrap();
        let rtk_carrier = cast_rtk_carrier(carrier);

        // try to gather remote observation
        if let Some(remote) = base_station.observe(*t, *sv, carrier) {
            remote_observations.push(remote);
        }

                    if observable.is_pseudorange_observable() {
                        if let Some(obs) = observations
                            .iter_mut()
                            .filter(|ob| ob.carrier == rtk_carrier)
                            .reduce(|k, _| k)
                        {
                            obs.set_pseudo_range(data.obs);
                        } else {
                            observations.push(Observation::pseudo_range(
                                rtk_carrier,
                                data.obs,
                                data.snr.map(|snr| snr.into()),
                            ));
                        }
                    } else if observable.is_phase_observable() {
                        let lambda = carrier.wavelength();
                        if let Some(obs) = observations
                            .iter_mut()
                            .filter(|ob| ob.carrier == rtk_carrier)
                            .reduce(|k, _| k)
                        {
                            obs.set_ambiguous_phase_range(data.obs * lambda);
                        } else {
                            observations.push(Observation::ambiguous_phase_range(
                                rtk_carrier,
                                data.obs * lambda,
                                data.snr.map(|snr| snr.into()),
                            ));
                        }
                    } else if observable.is_doppler_observable() {
                        if let Some(obs) = observations
                            .iter_mut()
                            .filter(|ob| ob.carrier == rtk_carrier)
                            .reduce(|k, _| k)
                        {
                            obs.set_doppler(data.obs);
                        } else {
                            observations.push(Observation::doppler(
                                rtk_carrier,
                                data.obs,
                                data.snr.map(|snr| snr.into()),
                            ));
                        }
                    }
                }
            }

        // create [Candidate]
        let mut candidate = Candidate::new(*sv, *t, observations.clone());

            // customization: clock corr
            match clock.next_clock_at(*t, *sv) {
                Some(dt) => {
                    candidate.set_clock_correction(dt);
                },
                None => {
                    error!("{} ({}) - no clock correction available", *t, *sv);
                },
            }
            // customization: TGD
            if let Some((_, _, eph)) = eph.borrow_mut().select(*t, *sv) {
                if let Some(tgd) = eph.tgd() {
                    debug!("{} ({}) - tgd: {}", *t, *sv, tgd);
                    candidate.set_group_delay(tgd);
                }
            }
            // customization: Tropo
            // TODO (Meteo)
            let tropo = TropoComponents::Unknown;
            candidate.set_tropo_components(tropo);

            // customization: Iono
            match ctx.data.brdc_navigation() {
                Some(brdc) => {
                    if let Some(model) = kb_model(brdc, *t) {
                        candidate.set_iono_components(IonoComponents::KbModel(model));
                    } else if let Some(model) = ng_model(brdc, *t) {
                        candidate.set_iono_components(IonoComponents::NgModel(model));
                    } else if let Some(model) = bd_model(brdc, *t) {
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
            if !remote_observations.is_empty() {
                candidate.set_remote_observations(remote_observations);
            }
            candidates.push(candidate);
        }

        match solver.resolve(*t, &candidates) {
            Ok((t, pvt)) => {
                solutions.insert(t, pvt);
            },
            Err(e) => warn!("{} : pvt solver error \"{}\"", t, e),
        }
    }

    solutions
}
