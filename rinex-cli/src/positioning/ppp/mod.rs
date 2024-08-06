//! PPP solver
use crate::{
    cli::Context,
    positioning::{
        bd_model,
        cast_rtk_carrier,
        kb_model,
        ng_model, //tropo_components,
        ClockStateProvider,
        EphemerisSource,
    },
};

use std::{cell::RefCell, collections::BTreeMap};

use rinex::{carrier::Carrier, observation::LliFlags};

mod report;
pub use report::Report;

pub mod post_process;

use gnss_rtk::prelude::{
    BaseStation, Candidate, Epoch, IonoComponents, Observation, OrbitalStateProvider, PVTSolution,
    Solver, TropoComponents,
};

pub fn resolve<'a, 'b, CK: ClockStateProvider, O: OrbitalStateProvider, B: BaseStation>(
    ctx: &Context,
    mut eph: &'a RefCell<EphemerisSource<'b>>,
    mut clock: CK,
    mut solver: Solver<O, B>,
    // rx_lat_ddeg: f64,
) -> BTreeMap<Epoch, PVTSolution> {
    let mut solutions: BTreeMap<Epoch, PVTSolution> = BTreeMap::new();

    // infaillible, at this point
    let obs_data = ctx.data.observation().unwrap();
    let nav_data = ctx.data.brdc_navigation().unwrap();
    // let meteo_data = ctx.data.meteo(); //TODO

    for ((t, flag), (_clk, vehicles)) in obs_data.observation() {
        let mut candidates = Vec::<Candidate>::with_capacity(4);

        if !flag.is_ok() {
            // TODO: handle
            warn!("{}: aborting epoch on {} event", t, flag);
            continue;
        }

        for (sv, observations) in vehicles {
            let clock_corr = match clock.next_clock_at(*t, *sv) {
                Some(dt) => dt,
                None => {
                    error!("{} ({}) - no clock correction available", *t, *sv);
                    continue;
                },
            };

            let tgd = if let Some((_, _, eph)) = eph.borrow_mut().select(*t, *sv) {
                eph.tgd()
            } else {
                None
            };

            if let Some(tgd) = tgd {
                debug!("{} ({}) - tgd: {}", *t, *sv, tgd);
            }

            let mut rtk_obs = Vec::<Observation>::new();

            for (observable, data) in observations {
                if let Some(lli) = data.lli {
                    if lli != LliFlags::OK_OR_UNKNOWN {
                        // TODO: manage those events
                        warn!("lli not_ok: {}({}): {:?}", t, sv, lli);
                    }
                }
                if let Ok(carrier) = Carrier::from_observable(sv.constellation, observable) {
                    let rtk_carrier = cast_rtk_carrier(carrier);

                    if observable.is_pseudorange_observable() {
                        if let Some(obs) = rtk_obs
                            .iter_mut()
                            .filter(|ob| ob.carrier == rtk_carrier)
                            .reduce(|k, _| k)
                        {
                            obs.pseudo = Some(data.obs);
                        } else {
                            rtk_obs.push(Observation {
                                carrier: rtk_carrier,
                                snr: data.snr.map(|snr| snr.into()),
                                phase: None,
                                doppler: None,
                                ambiguity: None,
                                pseudo: Some(data.obs),
                            });
                        }
                    } else if observable.is_phase_observable() {
                        let lambda = carrier.wavelength();
                        if let Some(obs) = rtk_obs
                            .iter_mut()
                            .filter(|ob| ob.carrier == rtk_carrier)
                            .reduce(|k, _| k)
                        {
                            obs.phase = Some(data.obs * lambda);
                        } else {
                            rtk_obs.push(Observation {
                                carrier: rtk_carrier,
                                snr: data.snr.map(|snr| snr.into()),
                                pseudo: None,
                                doppler: None,
                                ambiguity: None,
                                phase: Some(data.obs * lambda),
                            });
                        }
                    } else if observable.is_doppler_observable() {
                        if let Some(obs) = rtk_obs
                            .iter_mut()
                            .filter(|ob| ob.carrier == rtk_carrier)
                            .reduce(|k, _| k)
                        {
                            obs.doppler = Some(data.obs);
                        } else {
                            rtk_obs.push(Observation {
                                carrier: rtk_carrier,
                                snr: data.snr.map(|snr| snr.into()),
                                pseudo: None,
                                phase: None,
                                ambiguity: None,
                                doppler: Some(data.obs),
                            });
                        }
                    }
                }
            }
            let iono_components = if let Some(model) = kb_model(nav_data, *t) {
                IonoComponents::KbModel(model)
            } else if let Some(model) = ng_model(nav_data, *t) {
                IonoComponents::NgModel(model)
            } else if let Some(model) = bd_model(nav_data, *t) {
                IonoComponents::BdModel(model)
            } else {
                //TODO STEC/IONEX
                IonoComponents::Unknown
            };
            let candidate = Candidate::new(
                *sv,
                *t,
                clock_corr,
                Default::default(),
                rtk_obs.clone(),
                iono_components,
                TropoComponents::Unknown, // TODO: meteo
            );
            candidates.push(candidate);
        }

        match solver.resolve(*t, &candidates) {
            Ok((t, pvt)) => {
                debug!("{} : {:?}", t, pvt);
                solutions.insert(t, pvt);
            },
            Err(e) => warn!("{} : pvt solver error \"{}\"", t, e),
        }
    }

    solutions
}
