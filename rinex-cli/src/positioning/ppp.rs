use crate::cli::Context;
use rtk::prelude::Solver;

pub fn resolve(ctx: &Context, solver: Solver<APC, I>) -> BTreeMap<Epoch, PVTSolution> {
    let mut solutions: BTreeMap<Epoch, PVTSolution> = BTreeMap::new();

    for ((t, flag), (clk, vehicles)) in obs_data.observation() {
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

            let (toe, sv_eph) = sv_eph.unwrap();

            /*
             * Prefer SP3 for clock state (if any),
             * otherwise, use brdc
             */
            let clock_state = match sp3_has_clock {
                true => {
                    let sp3 = sp3_data.unwrap();
                    if let Some(_clk) = sp3
                        .sv_clock()
                        .filter_map(|(sp3_t, sp3_sv, clk)| {
                            if sp3_t == *t && sp3_sv == *sv {
                                Some(clk * 1.0E-6)
                            } else {
                                None
                            }
                        })
                        .reduce(|clk, _| clk)
                    {
                        let clock_state = sv_eph.sv_clock();
                        Vector3::new(clock_state.0, 0.0_f64, 0.0_f64)
                    } else {
                        /*
                         * SP3 preference: abort on missing Epochs
                         */
                        //continue ;
                        let clock_state = sv_eph.sv_clock();
                        Vector3::new(clock_state.0, clock_state.1, clock_state.2)
                    }
                },
                false => {
                    let clock_state = sv_eph.sv_clock();
                    Vector3::new(clock_state.0, clock_state.1, clock_state.2)
                },
            };

            let clock_corr = Ephemeris::sv_clock_corr(
                *sv,
                (clock_state[0], clock_state[1], clock_state[2]),
                *t,
                toe,
            );

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
        let zwd_zdd = tropo_components(meteo_data, *t, lat_ddeg);

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
