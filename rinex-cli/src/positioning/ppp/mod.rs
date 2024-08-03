//! PPP solver
use crate::{
    cli::Context,
    positioning::{
        bd_model,
        cast_rtk_carrier,
        kb_model,
        ng_model, //tropo_components,
        ClockStateProvider,
    },
};
use std::collections::BTreeMap;

use rinex::{carrier::Carrier, observation::LliFlags};

mod report;
pub use report::Report;

pub mod post_process;

use gnss_rtk::prelude::{
    BaseStation, Candidate, Epoch, IonosphereBias, Observation, OrbitalStateProvider, PVTSolution,
    Solver, TroposphereBias,
};

pub fn resolve<CK: ClockStateProvider, O: OrbitalStateProvider, B: BaseStation>(
    ctx: &Context,
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
            // TODO: handle these invalid Epochs
            warn!("{}: (unhandled) rx event: {}", t, flag);
            continue;
        }

        // /*
        //  * store possibly provided clk state estimator,
        //  * so we can compare ours to this one later
        //  */
        // if let Some(clk) = clk {
        //     provided_clk.insert(*t, *clk);
        // }

        for (sv, observations) in vehicles {
            // TODO/NB: we need to be able to operate without Ephemeris source
            //          to support pure rtk
            let clock_corr = match clock.next_clock_at(*t, *sv) {
                Some(dt) => dt,
                None => {
                    error!("{} ({}) - no clock correction available", *t, *sv);
                    continue;
                },
            };

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
            let candidate =
                Candidate::new(*sv, *t, clock_corr, Default::default(), rtk_obs.clone());
            candidates.push(candidate);
        }

        // grab possible tropo components
        // let zwd_zdd = tropo_components(meteo_data, *t, rx_lat_ddeg);

        let iono_bias = IonosphereBias {
            kb_model: kb_model(nav_data, *t),
            bd_model: bd_model(nav_data, *t),
            ng_model: ng_model(nav_data, *t),
            stec_meas: None, //TODO
        };

        let tropo_bias = TroposphereBias {
            total: None,   //TODO
            zwd_zdd: None, //TODO
        };

        match solver.resolve(*t, &candidates, &iono_bias, &tropo_bias) {
            Ok((t, pvt)) => {
                debug!("{} : {:?}", t, pvt);
                solutions.insert(t, pvt);
            },
            Err(e) => warn!("{} : pvt solver error \"{}\"", t, e),
        }
    }

    solutions
}
