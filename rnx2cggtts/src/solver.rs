use crate::Cli;

use gnss::prelude::{Constellation, SV};

use rinex::{
    carrier::Carrier,
    navigation::Ephemeris,
    prelude::{Observable, Rinex, RnxContext},
};

use rtk::{
    model::TropoComponents,
    prelude::{
        AprioriPosition, Candidate, Config, Duration, Epoch, InterpolationResult, Mode,
        PVTSolution, PVTSolutionType, PseudoRange, Solver, TimeScale,
    },
    Vector3D,
};

use cggtts::{
    prelude::{CommonViewClass, Track, TrackData},
    track::{FitData, Scheduler},
};

//use statrs::statistics::Statistics;
use map_3d::{ecef2geodetic, Ellipsoid};

use std::collections::HashMap;
use std::str::FromStr;
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

pub fn resolve(ctx: &mut RnxContext, cli: &Cli) -> Result<Vec<Track>, Error> {
    // cli customizations
    let trk_duration = cli.tracking_duration();
    let trk_duration_s = trk_duration.total_nanoseconds() as f64 * 1.0E-9;
    info!("tracking duration: {}", trk_duration);

    let single_sv: Option<SV> = match cli.single_sv() {
        Some(sv) => {
            info!("tracking {}", sv);
            Some(sv)
        },
        None => None,
    };

    // custom strategy
    let rtk_mode = match cli.spp() {
        true => {
            info!("spp position solver");
            Mode::SPP
        },
        false => Mode::SPP, //TODO
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

    let dominant_sample_rate = obs_data
        .dominant_sample_rate()
        .expect("RNX2CGGTTS requires steady RINEX observations");

    let first_epoch = obs_data
        .first_epoch()
        .expect("RNX2CGGTTS requires RINEX observations to be populated");

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
                    let (elev, azi) = Ephemeris::elevation_azimuth((x, y, z), apriori_ecef_wgs84);
                    Some(InterpolationResult {
                        sky_pos: (x, y, z).into(),
                        elevation: Some(elev),
                        azimuth: Some(azi),
                    })
                } else {
                    // debug!("{:?} ({}): sp3 interpolation failed", t, sv);
                    if let Some((x, y, z)) = nav_data.sv_position_interpolate(sv, t, order) {
                        let (x, y, z) = (x * 1.0E3, y * 1.0E3, z * 1.0E3);
                        let (elev, azi) =
                            Ephemeris::elevation_azimuth((x, y, z), apriori_ecef_wgs84);
                        Some(InterpolationResult {
                            sky_pos: (x, y, z).into(),
                            elevation: Some(elev),
                            azimuth: Some(azi),
                        })
                    } else {
                        // debug!("{:?} ({}): nav interpolation failed", t, sv);
                        None
                    }
                }
            } else {
                if let Some((x, y, z)) = nav_data.sv_position_interpolate(sv, t, order) {
                    let (x, y, z) = (x * 1.0E3, y * 1.0E3, z * 1.0E3);
                    let (elev, azi) = Ephemeris::elevation_azimuth((x, y, z), apriori_ecef_wgs84);
                    Some(InterpolationResult {
                        sky_pos: (x, y, z).into(),
                        elevation: Some(elev),
                        azimuth: Some(azi),
                    })
                } else {
                    // debug!("{:?} ({}): nav interpolation failed", t, sv);
                    None
                }
            }
        },
    )?;

    // CGGTTS specifics
    let mut tracks = Vec::<Track>::new();
    let mut sched = Scheduler::new(first_epoch.in_time_scale(TimeScale::UTC), trk_duration);

    for ((t, flag), (clk, vehicles)) in obs_data.observation() {
        /*
         * we only consider "OK" Epochs
         */
        if !flag.is_ok() {
            continue;
        }
        // resolve a PVT
        let mut candidates = Vec::<Candidate>::with_capacity(4);

        for (sv, observations) in vehicles {
            // form PVT candidate
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

            for (observable, data) in observations {
                //TODO: we can only latch "L1" observables at the moment..
                if observable.to_string().starts_with("C1") {
                    if observable.is_pseudorange_observable() {
                        pseudo_range.push(PseudoRange {
                            value: data.obs,
                            frequency: Carrier::from_str("L1").unwrap().frequency(), //TODO L2, L5..
                        });
                    }
                }
            }

            if let Ok(candidate) =
                Candidate::new(*sv, *t, clock_state, clock_corr, None, pseudo_range.clone())
            {
                candidates.push(candidate);
            } else {
                warn!("{:?}: failed to form {} candidate", t, sv);
            }
        }

        // resolve PVT
        let tropo_components = tropo_components(meteo_data, *t, lat_ddeg);

        match solver.resolve(
            *t,
            PVTSolutionType::TimeOnly,
            candidates.clone(),
            tropo_components,
            None,
        ) {
            Ok((t, pvt_solution)) => {
                debug!("{:?} : {:?}", t, pvt_solution);

                let fitdata = FitData {
                    refsv: pvt_solution.dt,
                    refsys: 0.0_f64, //TODO
                    mdtr: pvt_solution.tropo,
                    mdio: pvt_solution.iono.modeled,
                    msio: pvt_solution.iono.measured,
                    azimuth: {
                        if single_sv.is_some() {
                            candidates[0].azimuth.unwrap()
                        } else {
                            9999.0_f64
                        }
                    },
                    elevation: {
                        if single_sv.is_some() {
                            candidates[0].elevation.unwrap()
                        } else {
                            999.0_f64
                        }
                    },
                };

                let ioe = 0; //TODO
                match sched.latch_measurements(t, fitdata, ioe) {
                    Err(e) => {
                        warn!("{:?} - track fitting error: \"{}\"", t, e);
                        sched.reset(t);
                    },
                    Ok(None) => info!("{:?} - {} until next track", t, sched.time_to_next_track(t)),
                    Ok(Some(((trk_elev, trk_azi), trk_data))) => {
                        debug!("{:?} - formed new track", t);

                        let track = match single_sv {
                            Some(sv) => {
                                /* we're in focused common view */
                                Track::new_sv(
                                    sv,
                                    t.in_time_scale(TimeScale::UTC),
                                    trk_duration,
                                    CommonViewClass::SingleChannel,
                                    trk_elev,
                                    trk_azi,
                                    trk_data,
                                    None,  //TODO "iono": once L2/L5 unlocked,
                                    99,    // TODO "rcvr_channel"
                                    "C1C", //TODO: verify this please
                                )
                            },
                            None => {
                                /* melting pot */
                                Track::new_melting_pot(
                                    t.in_time_scale(TimeScale::UTC),
                                    trk_duration,
                                    CommonViewClass::MultiChannel,
                                    trk_elev,
                                    trk_azi,
                                    trk_data,
                                    None,  //TODO "iono": once L2/L5 unlocked,
                                    99,    // TODO "rcvr_channel"
                                    "C1C", //TODO: verify this please
                                )
                            },
                        };

                        tracks.push(track);
                    },
                }
            },
            Err(e) => warn!("{:?} - pvt resolution error \"{}\"", t, e),
        }
    }

    Ok(tracks)
}
