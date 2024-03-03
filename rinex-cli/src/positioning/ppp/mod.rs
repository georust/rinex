//! PPP solver
use crate::cli::Context;
use crate::positioning::{bd_model, kb_model, ng_model, tropo_components};
use rinex::carrier::Carrier;
use rinex::navigation::Ephemeris;
use rinex::prelude::SV;
use std::collections::BTreeMap;

mod post_process;
pub use post_process::{post_process, Error as PostProcessingError};

use rtk::prelude::{
    Candidate, Epoch, InterpolationResult, IonosphericBias, Observation, PVTSolution,
    PVTSolutionType, Solver, TroposphericBias, Vector3,
};

pub fn resolve<APC, I>(
    ctx: &Context,
    mut solver: Solver<APC, I>,
    rx_lat_ddeg: f64,
) -> BTreeMap<Epoch, PVTSolution>
where
    APC: Fn(Epoch, SV, f64) -> Option<(f64, f64, f64)>,
    I: Fn(Epoch, SV, usize) -> Option<InterpolationResult>,
{
    let mut solutions: BTreeMap<Epoch, PVTSolution> = BTreeMap::new();

    // infaillible, at this point
    let obs_data = ctx.data.obs_data().unwrap();
    let nav_data = ctx.data.nav_data().unwrap();
    let meteo_data = ctx.data.meteo_data();

    let clk_data = ctx.data.clk_data();
    let has_clk_data = clk_data.is_some();

    let sp3_data = ctx.data.sp3_data();

    let sp3_has_clock = if has_clk_data {
        false // always prefer clk data
    } else {
        match sp3_data {
            Some(sp3) => sp3.sv_clock().count() > 0,
            None => false,
        }
    };

    for ((t, flag), (_clk, vehicles)) in obs_data.observation() {
        let mut candidates = Vec::<Candidate>::with_capacity(4);

        if !flag.is_ok() {
            /* we only consider "OK" epochs" */
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
                warn!("{:?} ({}) : undetermined ephemeris", t, sv);
                continue; // can't proceed further
            }

            // determine TOE
            let (toe, sv_eph) = sv_eph.unwrap();
            /*
             * Clock state
             *   1. Prefer CLK product
             *   2. Prefer SP3 product
             *   3. Radio last option: always feasible
             */
            let clock_state = if has_clk_data {
                let clk = clk_data.unwrap();

                if let Some((_, profile)) = clk.sv_embedded_clock_interpolate(*t, *sv) {
                    (
                        profile.bias,
                        profile.drift.unwrap_or(0.0),
                        profile.drift_change.unwrap_or(0.0),
                    )
                } else {
                    /*
                     * do not interpolate other products: abort
                     */
                    continue;
                }
            } else if sp3_has_clock {
                unimplemented!("unhandled case");
            } else {
                sv_eph.sv_clock() // BRDC case
            };
            // determine clock correction
            let clock_corr = Ephemeris::sv_clock_corr(*sv, clock_state, *t, toe);

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
                        phases.push(Observation {
                            frequency,
                            snr: { data.snr.map(|snr| snr.into()) },
                            value: data.obs,
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

            let clock_state = Vector3::new(clock_state.0, clock_state.1, clock_state.2);

            if let Ok(candidate) = Candidate::new(
                *sv,
                *t,
                clock_state,
                clock_corr,
                codes.clone(),
                phases.clone(),
                dopplers.clone(),
            ) {
                candidates.push(candidate);
            } else {
                warn!("{:?}: failed to form {} candidate", t, sv);
            }
        }

        // grab possible tropo components
        let zwd_zdd = tropo_components(meteo_data, *t, rx_lat_ddeg);

        let iono_bias = IonosphericBias {
            kb_model: kb_model(nav_data, *t),
            bd_model: bd_model(nav_data, *t),
            ng_model: ng_model(nav_data, *t),
            stec_meas: None, //TODO
        };

        let tropo_bias = TroposphericBias {
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
