//! PPP solver
use crate::cli::Context;
use crate::positioning::{bd_model, kb_model, ng_model, tropo_components};
use std::collections::BTreeMap;

use rinex::{
    carrier::Carrier,
    prelude::{Duration, SV},
};

mod post_process;
pub use post_process::{post_process, Error as PostProcessingError};

use rtk::prelude::{
    Candidate, Epoch, InterpolationResult, IonosphereBias, Observation, PVTSolution,
    PVTSolutionType, Solver, TroposphereBias,
};

use super::interp::TimeInterpolator;

pub fn resolve<I>(
    ctx: &Context,
    mut solver: Solver<I>,
    rx_lat_ddeg: f64,
) -> BTreeMap<Epoch, PVTSolution>
where
    I: Fn(Epoch, SV, usize) -> Option<InterpolationResult>,
{
    let mut solutions: BTreeMap<Epoch, PVTSolution> = BTreeMap::new();

    // infaillible, at this point
    let obs_data = ctx.data.observation().unwrap();
    let nav_data = ctx.data.brdc_navigation().unwrap();

    let clk_data = ctx.data.clock();
    let meteo_data = ctx.data.meteo();

    let sp3_has_clock = ctx.data.sp3_has_clock();
    if clk_data.is_none() && sp3_has_clock {
        if let Some(sp3) = ctx.data.sp3() {
            warn!("Using clock states defined in SP3 file: CLK product should be prefered");
            if sp3.epoch_interval >= Duration::from_seconds(300.0) {
                warn!("interpolating clock states from low sample rate SP3 will most likely introduce errors");
            }
        }
    }

    let mut interp = TimeInterpolator::from_ctx(&ctx);
    debug!("Clock interpolator created");

    for ((t, flag), (_clk, vehicles)) in obs_data.observation() {
        let mut candidates = Vec::<Candidate>::with_capacity(4);

        if !flag.is_ok() {
            /* we only consider _valid_ epochs" */
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
            let sv_eph = nav_data.sv_ephemeris(*sv, *t);
            if sv_eph.is_none() {
                error!("{:?} ({}) : undetermined ephemeris", t, sv);
                continue; // can't proceed further
            }

            // determine TOE
            let (_toe, sv_eph) = sv_eph.unwrap();
            let clock_corr = match interp.next_at(*t, *sv) {
                Some(dt) => dt,
                None => {
                    error!("{:?} ({}) - failed to determine clock correction", *t, *sv);
                    continue;
                },
            };

            let mut codes = Vec::<Observation>::new();
            let mut phases = Vec::<Observation>::new();
            let mut dopplers = Vec::<Observation>::new();

            for (observable, data) in observations {
                if let Ok(carrier) = Carrier::from_observable(sv.constellation, observable) {
                    let frequency = carrier.frequency();

                    if observable.is_pseudorange_observable() {
                        codes.push(Observation {
                            frequency,
                            snr: { data.snr.map(|snr| snr.into()) },
                            value: data.obs,
                        });
                    } else if observable.is_phase_observable() {
                        let lambda = carrier.wavelength();
                        phases.push(Observation {
                            frequency,
                            snr: { data.snr.map(|snr| snr.into()) },
                            value: data.obs * lambda,
                        });
                    } else if observable.is_doppler_observable() {
                        dopplers.push(Observation {
                            frequency,
                            snr: { data.snr.map(|snr| snr.into()) },
                            value: data.obs,
                        });
                    }
                }
            }
            let candidate = Candidate::new(
                *sv,
                *t,
                clock_corr,
                sv_eph.tgd(),
                codes.clone(),
                phases.clone(),
                dopplers.clone(),
            );
            candidates.push(candidate);
        }

        // grab possible tropo components
        let zwd_zdd = tropo_components(meteo_data, *t, rx_lat_ddeg);

        let iono_bias = IonosphereBias {
            kb_model: kb_model(nav_data, *t),
            bd_model: bd_model(nav_data, *t),
            ng_model: ng_model(nav_data, *t),
            stec_meas: None, //TODO
        };

        let tropo_bias = TroposphereBias {
            total: None, //TODO
            zwd_zdd,
        };

        match solver.resolve(
            *t,
            PVTSolutionType::PositionVelocityTime,
            candidates,
            &iono_bias,
            &tropo_bias,
        ) {
            Ok((t, pvt)) => {
                debug!("{:?} : {:?}", t, pvt);
                solutions.insert(t, pvt);
            },
            Err(e) => warn!("{:?} : pvt solver error \"{}\"", t, e),
        }
    }

    solutions
}
