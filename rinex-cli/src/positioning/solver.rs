use crate::Cli;
//use statrs::statistics::Statistics;

use gnss::prelude::Constellation; // SV};
use rinex::carrier::Carrier;
use rinex::navigation::Ephemeris;
use rinex::prelude::{Observable, Rinex, RnxContext};

use rtk::prelude::{
    AprioriPosition, BdModel, Candidate, Config, Duration, Epoch, InterpolatedPosition, InterpolationResult,
    IonosphericBias, KbModel, Mode, NgModel, Observation, PVTSolution, PVTSolutionType, Solver,
    TroposphericBias, Vector3,
};

use map_3d::{ecef2geodetic, Ellipsoid};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("solver error")]
    SolverError(#[from] rtk::Error),
    #[error("missing observations")]
    MissingObservationData,
    #[error("missing brdc navigation")]
    MissingBroadcastNavigationData,
    #[error("positioning requires overlapped SP3 data at the moment")]
    MissingSp3Data,
    #[error("undefined apriori position")]
    UndefinedAprioriPosition,
}

fn tropo_components(meteo: Option<&Rinex>, t: Epoch, lat_ddeg: f64) -> Option<(f64, f64)> {
    const MAX_LATDDEG_DELTA: f64 = 15.0;
    let max_dt = Duration::from_hours(24.0);
    let rnx = meteo?;
    let meteo = rnx.header.meteo.as_ref().unwrap();

    let delays: Vec<(Observable, f64)> = meteo
        .sensors
        .iter()
        .filter_map(|s| match s.observable {
            Observable::ZenithDryDelay => {
                let (x, y, z, _) = s.position?;
                let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                if (lat - lat_ddeg).abs() < MAX_LATDDEG_DELTA {
                    let value = rnx
                        .zenith_dry_delay()
                        .filter(|(t_sens, _)| (*t_sens - t).abs() < max_dt)
                        .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
                    let (_, value) = value?;
                    debug!("{:?} lat={} zdd {}", t, lat_ddeg, value);
                    Some((s.observable.clone(), value))
                } else {
                    None
                }
            },
            Observable::ZenithWetDelay => {
                let (x, y, z, _) = s.position?;
                let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                if (lat - lat_ddeg).abs() < MAX_LATDDEG_DELTA {
                    let value = rnx
                        .zenith_wet_delay()
                        .filter(|(t_sens, _)| (*t_sens - t).abs() < max_dt)
                        .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
                    let (_, value) = value?;
                    debug!("{:?} lat={} zdd {}", t, lat_ddeg, value);
                    Some((s.observable.clone(), value))
                } else {
                    None
                }
            },
            _ => None,
        })
        .collect();

    if delays.len() < 2 {
        None
    } else {
        let zdd = delays
            .iter()
            .filter_map(|(obs, value)| {
                if obs == &Observable::ZenithDryDelay {
                    Some(*value)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
            .unwrap();

        let zwd = delays
            .iter()
            .filter_map(|(obs, value)| {
                if obs == &Observable::ZenithWetDelay {
                    Some(*value)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
            .unwrap();

        Some((zwd, zdd))
    }
}

fn kb_model(nav: &Rinex, t: Epoch) -> Option<KbModel> {
    let kb_model = nav
        .klobuchar_models()
        .min_by_key(|(t_i, _, _)| (t - *t_i).abs());

    match kb_model {
        Some((_, sv, kb_model)) => {
            Some(KbModel {
                h_km: {
                    match sv.constellation {
                        Constellation::BeiDou => 375.0,
                        // we only expect GPS or BDS here,
                        // badly formed RINEX will generate errors in the solutions
                        _ => 350.0,
                    }
                },
                alpha: kb_model.alpha,
                beta: kb_model.beta,
            })
        },
        None => None,
    }
}

fn bd_model(nav: &Rinex, t: Epoch) -> Option<BdModel> {
    nav.bdgim_models()
        .min_by_key(|(t_i, _)| (t - *t_i).abs())
        .map(|(_, model)| BdModel { alpha: model.alpha })
}

fn ng_model(nav: &Rinex, t: Epoch) -> Option<NgModel> {
    nav.nequick_g_models()
        .min_by_key(|(t_i, _)| (t - *t_i).abs())
        .map(|(_, model)| NgModel { a: model.a })
}

pub fn solver(ctx: &mut RnxContext, cli: &Cli) -> Result<BTreeMap<Epoch, PVTSolution>, Error> {
    // custom strategy
    let mode = cli.solver_mode().unwrap(); // infaillible

    match mode {
        Mode::SPP => info!("single point positioning"),
        Mode::LSQSPP => info!("recursive lsq single point positioning"),
        Mode::PPP => info!("precise point positioning"),
    };

    // parse custom config, if any
    let cfg = match cli.config() {
        Some(cfg) => cfg,
        None => Config::default(mode),
    };

    let pos = match cli.manual_position() {
        Some(pos) => pos,
        None => ctx
            .ground_position()
            .ok_or(Error::UndefinedAprioriPosition)?,
    };

    let apriori_ecef = pos.to_ecef_wgs84();
    let apriori = Vector3::<f64>::new(apriori_ecef.0, apriori_ecef.1, apriori_ecef.2);
    let apriori = AprioriPosition::from_ecef(apriori);

    let lat_ddeg = apriori.geodetic[0];

    // print config to be used
    info!("{:#?}", cfg);

    let obs_data = match ctx.obs_data() {
        Some(data) => data,
        None => {
            return Err(Error::MissingObservationData);
        },
    };

    let nav_data = match ctx.nav_data() {
        Some(data) => data,
        None => {
            return Err(Error::MissingBroadcastNavigationData);
        },
    };

    let sp3_data = ctx.sp3_data();
    let sp3_has_clock = match sp3_data {
        Some(sp3) => sp3.sv_clock().count() > 0,
        None => false,
    };

    let meteo_data = ctx.meteo_data();

    let mut solver = Solver::new(
        mode,
        apriori,
        &cfg,
        /* state vector interpolator */
        |t, sv, order| {
            /* SP3 source is prefered */
            if let Some(sp3) = sp3_data {
                if let Some((x, y, z)) = sp3.sv_position_interpolate(sv, t, order) {
                    let (x, y, z) = (x * 1.0E3, y * 1.0E3, z * 1.0E3);
                    let (elevation, azimuth) =
                        Ephemeris::elevation_azimuth((x, y, z), apriori_ecef);
                    Some(InterpolationResult {
                        azimuth,
                        elevation,
                        velocity: None,
                        position: InterpolatedPosition::MassCenter(Vector3::new(x, y, z)),
                    })
                } else {
                    // debug!("{:?} ({}): sp3 interpolation failed", t, sv);
                    if let Some((x, y, z)) = nav_data.sv_position_interpolate(sv, t, order) {
                        let (x, y, z) = (x * 1.0E3, y * 1.0E3, z * 1.0E3);
                        let (elevation, azimuth) =
                            Ephemeris::elevation_azimuth((x, y, z), apriori_ecef);
                        Some(InterpolationResult {
                            azimuth,
                            elevation,
                            velocity: None,
                            position: InterpolatedPosition::AntennaPhaseCenter(Vector3::new(x, y, z)),
                        })
                    } else {
                        // debug!("{:?} ({}): nav interpolation failed", t, sv);
                        None
                    }
                }
            } else {
                if let Some((x, y, z)) = nav_data.sv_position_interpolate(sv, t, order) {
                    let (x, y, z) = (x * 1.0E3, y * 1.0E3, z * 1.0E3);
                    let (elevation, azimuth) =
                        Ephemeris::elevation_azimuth((x, y, z), apriori_ecef);
                    Some(InterpolationResult {
                        azimuth,
                        elevation,
                        position: InterpolatedPosition::AntennaPhaseCenter(Vector3::new(x, y, z)),
                        velocity: None,
                    })
                } else {
                    // debug!("{:?} ({}): nav interpolation failed", t, sv);
                    None
                }
            }
        },
    )?;

    // resolved PVT solutions
    let mut solutions: BTreeMap<Epoch, PVTSolution> = BTreeMap::new();
    // possibly provided resolved T components (contained in RINEX)
    let mut provided_clk: HashMap<Epoch, f64> = HashMap::new();

    for ((t, flag), (clk, vehicles)) in obs_data.observation() {
        let mut candidates = Vec::<Candidate>::with_capacity(4);

        if !flag.is_ok() {
            /* we only consider "OK" epochs" */
            continue;
        }

        /*
         * store possibly provided clk state estimator,
         * so we can compare ours to this one later
         */
        if let Some(clk) = clk {
            provided_clk.insert(*t, *clk);
        }

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
                    if let Some(clk) = sp3.sv_clock()
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
            
            let clock_corr = Ephemeris::sv_clock_corr(*sv, (clock_state[0], clock_state[1], clock_state[2]), *t, toe);

            let mut codes = Vec::<Observation>::new();
            let mut phases = Vec::<Observation>::new();
            let mut dopplers = Vec::<Observation>::new();

            for (observable, data) in observations {
                if let Ok(carrier) = Carrier::from_observable(sv.constellation, observable) {
                    let frequency = carrier.frequency();

                    if observable.is_pseudorange_observable() {
                        codes.push(Observation {
                            frequency,
                            snr: {
                                if let Some(snr) = data.snr {
                                    Some(snr.into())
                                } else {
                                    None
                                }
                            },
                            value: data.obs,
                        });
                    } else if observable.is_phase_observable() {
                        phases.push(Observation {
                            frequency,
                            snr: {
                                if let Some(snr) = data.snr {
                                    Some(snr.into())
                                } else {
                                    None
                                }
                            },
                            value: data.obs,
                        });
                    } else if observable.is_doppler_observable() {
                        dopplers.push(Observation {
                            frequency,
                            snr: {
                                if let Some(snr) = data.snr {
                                    Some(snr.into())
                                } else {
                                    None
                                }
                            },
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

    Ok(solutions)
}
