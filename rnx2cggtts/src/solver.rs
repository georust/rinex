use crate::Cli;
use gnss::prelude::{Constellation, SV};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

use rinex::{
    carrier::Carrier,
    navigation::Ephemeris,
    prelude::{Observable, Rinex, RnxContext},
};

use rtk::{
    prelude::{
        AprioriPosition, Candidate, Config, Duration, Epoch, InterpolationResult, Mode,
        PVTSolutionType, PseudoRange, Solver, TimeScale, TropoComponents,
    },
    Vector3D,
};

use cggtts::{
    prelude::{CommonViewClass, Track},
    track::{FitData, GlonassChannel, SVTracker, Scheduler},
};

//use statrs::statistics::Statistics;
use map_3d::{ecef2geodetic, Ellipsoid};

#[derive(Debug, Error)]
pub enum Error {
    #[error("solver error")]
    SolverError(#[from] rtk::Error),
    #[error("missing observations")]
    MissingObservationData,
    #[error("missing brdc navigation")]
    MissingBroadcastNavigationData,
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
    // custom tracking duration
    let trk_duration = cli.tracking_duration();
    info!("tracking duration set to {}", trk_duration);

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

    let pos = match cli.manual_apc() {
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

    let dominant_sampling_period = obs_data
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
                    let (elevation, azimuth) =
                        Ephemeris::elevation_azimuth((x, y, z), apriori_ecef_wgs84);
                    Some(InterpolationResult {
                        azimuth,
                        elevation,
                        sky_pos: (x, y, z).into(),
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

    // CGGTTS specifics
    let mut tracks = Vec::<Track>::new();
    let mut sched = Scheduler::new(trk_duration);
    let mut next_release = Option::<Epoch>::None;
    let mut trk_midpoint = Option::<Epoch>::None;
    let mut trackers = HashMap::<SV, SVTracker>::new();

    let l1_frequency = Carrier::from_str("L1").unwrap().frequency(); //TODO L2, L5..

    for ((t, flag), (_clk, vehicles)) in obs_data.observation() {
        /*
         * we only consider "OK" Epochs
         */
        if !flag.is_ok() {
            continue;
        }

        // resolve a PVT for every single SV, at every single "t"
        let tropo_components = tropo_components(meteo_data, *t, lat_ddeg);

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
            let _snr = Vec::<f64>::new();
            let mut pseudo_range = Vec::<PseudoRange>::new();

            for (observable, data) in observations {
                //TODO: we can only latch "L1" observables at the moment..
                if observable.to_string().starts_with("C1") {
                    if observable.is_pseudorange_observable() {
                        pseudo_range.push(PseudoRange {
                            value: data.obs,
                            frequency: l1_frequency,
                        });
                    }
                }
            }

            let candidate =
                Candidate::new(*sv, *t, clock_state, clock_corr, None, pseudo_range.clone());

            if candidate.is_err() {
                warn!(
                    "{:?}: failed to form candidate {} : \"{}\"",
                    t,
                    sv,
                    candidate.err().unwrap()
                );
                continue;
            }

            let candidate = candidate.unwrap();
            match solver.resolve(
                *t,
                PVTSolutionType::TimeOnly, // Single SV opmode
                vec![candidate],           // Single SV opmode
                tropo_components,
                None,
            ) {
                Ok((t, mut pvt_solution)) => {
                    let pvt_data = pvt_solution.sv.get(sv).unwrap(); // infaillible

                    let azimuth = pvt_data.azimuth;
                    let elevation = pvt_data.elevation;

                    let refsys = pvt_solution.dt;
                    let refsv = pvt_solution.dt + clock_corr.to_seconds();

                    let mdtr = pvt_data.tropo.value().unwrap_or(0.0_f64); // MDTR evaluation
                                                                          // is always required in RNX2CGGTTS
                                                                          // this absorb a possibly disabled TROPO compensation
                                                                          // in the rtk solver.

                    let mdio = pvt_data.iono.modeled;
                    let msio = pvt_data.iono.measured;
                    debug!(
                        "{:?} : new {} PVT solution (elev={:.2}°, azi={:.2}°, REFSV={:.3E}, REFSYS={:.3E})",
                        t, sv, elevation, azimuth, refsv, refsys
                    );

                    let fitdata = FitData {
                        refsv,
                        refsys,
                        mdtr,
                        mdio,
                        msio,
                        azimuth,
                        elevation,
                    };

                    let mut tracker = match trackers.get_mut(sv) {
                        None => {
                            // initialize new tracker
                            trackers.insert(*sv, SVTracker::default());
                            trackers.get_mut(sv).unwrap()
                        },
                        Some(tracker) => tracker,
                    };

                    // verify buffer continuity
                    if !tracker.no_gaps(dominant_sampling_period) {
                        // on any discontinuity we need to reset
                        // that tracker. This will abort the ongoing track.
                        tracker.reset();
                        warn!("{:?} - discarding {} track due to data gaps", t, sv);

                        // push new measurement
                        tracker.latch_measurement(t, fitdata);
                        continue; // abort for this SV
                    }

                    if next_release.is_some() {
                        let next_release = next_release.unwrap();
                        let trk_midpoint = trk_midpoint.unwrap();

                        if t >= next_release {
                            /* time to release a track */
                            let ioe = 0; //TODO
                                         // latch last measurement
                            tracker.latch_measurement(t, fitdata);

                            match tracker.fit(
                                ioe,
                                trk_duration,
                                dominant_sampling_period,
                                trk_midpoint,
                            ) {
                                Ok(((trk_elev, trk_azi), trk_data)) => {
                                    info!(
                                        "{:?} - new {} track: elev {:.2}° - azi {:.2}° - REFSV {:.3E} REFSYS {:.3E}",
                                        t,
                                        sv,
                                        trk_elev,
                                        trk_azi,
                                        trk_data.refsv,
                                        trk_data.refsys
                                    );

                                    let track = match sv.constellation {
                                        Constellation::Glonass => {
                                            Track::new_glonass(
                                                *sv,
                                                next_release,
                                                trk_duration,
                                                CommonViewClass::SingleChannel,
                                                trk_elev,
                                                trk_azi,
                                                trk_data,
                                                None, //TODO "iono": once L2/L5 unlocked,
                                                0,    // TODO "rcvr_channel" > 0 if known
                                                GlonassChannel::default(), //TODO
                                                "C1C", //TODO
                                            )
                                        },
                                        _ => {
                                            Track::new(
                                                *sv,
                                                next_release,
                                                trk_duration,
                                                CommonViewClass::SingleChannel,
                                                trk_elev,
                                                trk_azi,
                                                trk_data,
                                                None,  //TODO "iono": once L2/L5 unlocked,
                                                0,     // TODO "rcvr_channel" > 0 if known
                                                "C1C", //TODO
                                            )
                                        },
                                    }; // match constellation
                                    tracks.push(track);
                                },
                                Err(e) => warn!("{:?} - track fitting error: \"{}\"", t, e),
                            } //.fit()

                            // reset so we start a new track
                            tracker.reset();
                        }
                        // time to release a track
                        else {
                            tracker.latch_measurement(t, fitdata);
                        }
                    }
                    //release.is_none()
                    else {
                        tracker.latch_measurement(t, fitdata);
                    }
                },
                Err(e) => {
                    warn!("{:?} - pvt resolution error \"{}\"", t, e);
                    /*
                     * Any PVT resolution failures would introduce a data gap
                     * which is incompatible with CGGTTS track fitting
                     */
                    if let Some(tracker) = trackers.get_mut(sv) {
                        tracker.reset();
                    }
                },
            } //.pvt resolve
        } //.sv()
        next_release = Some(sched.next_track_start(*t));
        trk_midpoint = Some(next_release.unwrap() - trk_duration / 2);
        info!("{:?} - {} until next track", t, next_release.unwrap() - *t);
    } //.observations()

    Ok(tracks)
}
