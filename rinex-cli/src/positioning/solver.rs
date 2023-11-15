use crate::Cli;
use statrs::statistics::Statistics;

// use gnss::prelude::{Constellation, SV};
use rinex::carrier::Carrier;
use rinex::navigation::Ephemeris;
use rinex::prelude::{Observable, Rinex, RnxContext};

use rtk::{
    prelude::{
        AprioriPosition, Candidate, Config, Duration, Epoch, InterpolationResult, IonosphericDelay,
        Mode, PVTSolution, PVTSolutionType, PseudoRange, Solver, TropoComponents,
    },
    Vector3D,
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

fn tropo_components(meteo: Option<&Rinex>, t: Epoch, lat_ddeg: f64) -> Option<TropoComponents> {
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
        Some(TropoComponents {
            zdd: {
                delays
                    .iter()
                    .filter_map(|(obs, value)| {
                        if obs == &Observable::ZenithDryDelay {
                            Some(*value)
                        } else {
                            None
                        }
                    })
                    .reduce(|k, _| k)
                    .unwrap()
            },
            zwd: {
                delays
                    .iter()
                    .filter_map(|(obs, value)| {
                        if obs == &Observable::ZenithWetDelay {
                            Some(*value)
                        } else {
                            None
                        }
                    })
                    .reduce(|k, _| k)
                    .unwrap()
            },
        })
    }
}

pub fn solver(ctx: &mut RnxContext, cli: &Cli) -> Result<BTreeMap<Epoch, PVTSolution>, Error> {
    // custom strategy
    let rtk_mode = match cli.spp() {
        true => {
            info!("single point positioning opmode");
            Mode::SPP
        },
        false => {
            info!("precise point positioning opmode");
            //TODO: verify feasiblity here, otherwise panic
            Mode::PPP
        },
    };

    // parse custom config, if any
    let cfg = match cli.config() {
        Some(cfg) => cfg,
        None => Config::default(rtk_mode),
    };

    let pos = match cli.manual_position() {
        Some(pos) => pos,
        None => ctx
            .ground_position()
            .ok_or(Error::UndefinedAprioriPosition)?,
    };

    let apriori_ecef_wgs84 = pos.to_ecef_wgs84();
    let apriori = AprioriPosition::from_ecef(apriori_ecef_wgs84.into());
    let lat_ddeg = apriori.geodetic.x;

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
    let meteo_data = ctx.meteo_data();

    let mut solver = Solver::new(
        rtk_mode,
        apriori,
        &cfg,
        /* state vector interpolator */
        |t, sv, order| {
            /* SP3 source is prefered */
            if let Some(sp3) = sp3_data {
                if let Some((x, y, z)) = sp3.sv_position_interpolate(sv, t, order) {
                    let (x, y, z) = (x * 1.0E3, y * 1.0E3, z * 1.0E3);
                    let (elevation, azimuth) =
                        Ephemeris::elevation_azimuth((x, y, z), apriori_ecef_wgs84);
                    Some(InterpolationResult {
                        sky_pos: (x, y, z).into(),
                        azimuth,
                        elevation,
                    })
                } else {
                    // debug!("{:?} ({}): sp3 interpolation failed", t, sv);
                    if let Some((x, y, z)) = nav_data.sv_position_interpolate(sv, t, order) {
                        let (x, y, z) = (x * 1.0E3, y * 1.0E3, z * 1.0E3);
                        let (elevation, azimuth) =
                            Ephemeris::elevation_azimuth((x, y, z), apriori_ecef_wgs84);
                        Some(InterpolationResult {
                            azimuth,
                            elevation,
                            sky_pos: (x, y, z).into(),
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
                        Ephemeris::elevation_azimuth((x, y, z), apriori_ecef_wgs84);
                    Some(InterpolationResult {
                        azimuth,
                        elevation,
                        sky_pos: (x, y, z).into(),
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

            let clock_state = sv_eph.sv_clock();
            let clock_corr = Ephemeris::sv_clock_corr(*sv, clock_state, *t, toe);
            let clock_state: Vector3D = clock_state.into();
            let mut snr = Vec::<f64>::new();
            let mut pseudo_range = Vec::<PseudoRange>::new();
            let mut modeled_ionod = Vec::<IonosphericDelay>::new();

            for (observable, data) in observations {
                if let Ok(carrier) = Carrier::from_observable(sv.constellation, observable) {
                    let frequency = carrier.frequency();

                    if observable.is_pseudorange_observable() {
                        pseudo_range.push(PseudoRange {
                            frequency,
                            value: data.obs,
                        });
                        if let Some(obs_snr) = data.snr {
                            snr.push(obs_snr.into());
                        }
                    }

                    /* IONOD */
                    if cfg.modeling.iono_delay {
                        /* inject possible models */
                        if let Some(time_delay) = nav_data.ionod_model(
                            *t,
                            sv,
                            sv_elevation,
                            sv_azim,
                            apriori.geodetic.x,
                            apriori.geodetic.y,
                            carrier,
                        ) {
                            modeled_ionod.push(IonosphericDelay {
                                frequency,
                                time_delay,
                            });
                        }
                        /* provide possible STEC */
                    }
                }
            }

            /* worst SNR over all used observations */
            let snr: Option<f64> = match snr.is_empty() {
                true => None,
                false => {
                    let snr = snr.min();
                    debug!("{:?} ({}): SNR : {:?}", t, sv, snr);
                    Some(snr)
                },
            };

            if let Ok(candidate) = Candidate::new(
                *sv,
                *t,
                clock_state,
                clock_corr,
                snr,
                pseudo_range.clone(),
                modeled_ionod,
            ) {
                candidates.push(candidate);
            } else {
                warn!("{:?}: failed to form {} candidate", t, sv);
            }
        }
        let tropo_components = tropo_components(meteo_data, *t, lat_ddeg);

        match solver.resolve(
            *t,
            PVTSolutionType::PositionVelocityTime,
            candidates,
            tropo_components,
            None,
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
