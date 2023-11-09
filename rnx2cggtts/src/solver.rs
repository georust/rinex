use crate::Cli;
use statrs::statistics::Statistics;

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
        PVTSolution, PVTSolutionType, PseudoRange, Solver,
    },
    Vector3D,
};

use cggtts::{
    prelude::{CommonViewClass, Track, TrackData},
    track::SkyTracker,
};

use map_3d::{ecef2geodetic, Ellipsoid};

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

    let mut t0 = Option::<Epoch>::None;
    let mut dsg = 0.0_f64;
    let mut tracker = SkyTracker::default().tracking_duration(trk_duration);
    let mut time_to_next = Option::<Duration>::None;
    let mut srsys = 0.0_f64;
    let mut refsys: Option<f64> = None;
    let (mut trk_azi, mut trk_elev) = (0.0_f64, 0.0_f64);
    let mut melting_pot = false;

    let trk_nb_avg =
        (trk_duration.total_nanoseconds() / dominant_sample_rate.total_nanoseconds()) as u32;

    // print more infos
    info!("using tracking duration: {}", trk_duration);

    for (index, ((t, flag), (clk, vehicles))) in obs_data.observation().enumerate() {
        /* synchronize sky tracker prior anything */
        if t0.is_none() {
            t0 = Some(tracker.next_track_start(*t));
        }

        let t0 = t0.unwrap();
        if *t < t0 {
            // Dropping first partial track
            trace!("{:?} - tracker is not synchronized", *t);
            continue;
        }

        /*
         * Tracker is now aligned & ""synchronized""
         * we only consider "OK" Epochs
         */
        if !flag.is_ok() {
            continue;
        }

        /* latch all PR observations */
        for (sv, observations) in vehicles {
            for (observable, data) in observations {
                //TODO: we only latch "L1" observables at the moment..
                if observable.to_string().starts_with("C1") {
                    if observable.is_pseudorange_observable() {
                        tracker.latch_data(*t, *sv, data.obs);
                    }
                }
            }
        }

        let time_to_next_track = tracker.time_to_next_track(*t);

        if let Some(prev_time_to_next) = time_to_next {
            // if time_to_next_track <= trk_duration /2 && prev_time_to_next > trk_duration /2 {
            //     /* mid track point */
            //     }
            // }

            if time_to_next_track >= prev_time_to_next {
                debug!("{:?} - releasing new track", *t);

                let mut candidates = Vec::<Candidate>::with_capacity(4);

                for (sv, _) in vehicles {
                    let tracker = tracker
                        .pool
                        .iter()
                        .filter_map(|(svnn, trk)| if svnn == sv { Some(trk) } else { None })
                        .reduce(|trk, _| trk);

                    if let Some(tracker) = tracker {
                        // 1. only continuously tracked vehicles will contribute
                        if tracker.n_avg != trk_nb_avg {
                            debug!(
                                "{:?} - {} ({}/{}) loss of sight, will not contribute",
                                *t, sv, tracker.n_avg, trk_nb_avg
                            );
                            continue;
                        }

                        // 2. Resolve
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

                        let mut pseudo_range = vec![PseudoRange {
                            value: tracker.pseudo_range,
                            //TODO: improve this once L2/L5.. are unlocked
                            frequency: Carrier::from_str("L1").unwrap().frequency(),
                        }];

                        if let Ok(candidate) = Candidate::new(
                            *sv,
                            *t,
                            clock_state,
                            clock_corr,
                            None,
                            pseudo_range.clone(),
                        ) {
                            candidates.push(candidate);
                        } else {
                            warn!("{:?}: failed to form {} candidate", t, sv);
                        }
                    }
                }

                let tropo_components = tropo_components(meteo_data, *t, lat_ddeg);

                match solver.resolve(
                    *t,
                    PVTSolutionType::TimeOnly,
                    candidates.clone(),
                    tropo_components,
                    None,
                ) {
                    Ok((t, pvt_solution)) => {
                        // form a CGGTTS track
                        debug!("{:?} : {:?}", t, pvt_solution);

                        if let Some(refsys) = refsys {
                            srsys = (pvt_solution.dt - refsys)
                                / (trk_duration.total_nanoseconds() as f64 * 1.0E-9);
                        }

                        refsys = Some(pvt_solution.dt);

                        let track = match melting_pot {
                            true => {
                                /* we're in focused common view */
                                Track::new_sv(
                                    candidates.clone()[0].sv,
                                    t,
                                    trk_duration,
                                    CommonViewClass::SingleChannel,
                                    trk_elev,
                                    trk_azi,
                                    TrackData {
                                        refsv: 9999999999.0,
                                        srsv: 99999.0,
                                        refsys: pvt_solution.dt,
                                        srsys, //TODO
                                        dsg,   //TODO
                                        ioe: 999,
                                        mdtr: 9999.0,
                                        smdt: 999.0,
                                        mdio: 9999.0,
                                        smdi: 999.0,
                                    },
                                    None,  //TODO "iono": once L2/L5 unlocked,
                                    99,    // TODO "rcvr_channel"
                                    "C1C", //TODO: verify this please
                                )
                            },
                            false => {
                                /* melting pot */
                                Track::new_melting_pot(
                                    t,
                                    trk_duration,
                                    CommonViewClass::MultiChannel,
                                    trk_elev,
                                    trk_azi,
                                    TrackData {
                                        refsv: 9999999999.0,
                                        srsv: 99999.0,
                                        refsys: pvt_solution.dt,
                                        srsys, //TODO
                                        dsg,   //TODO
                                        ioe: 999,
                                        mdtr: 9999.0,
                                        smdt: 999.0,
                                        mdio: 9999.0,
                                        smdi: 999.0,
                                    },
                                    None,  //TODO "iono": once L2/L5 unlocked,
                                    99,    // TODO "rcvr_channel"
                                    "C1C", //TODO: verify this please
                                )
                            },
                        };

                        tracks.push(track);
                    },
                    Err(e) => warn!("{:?} : {}", t, e),
                }

                // RESET the tracker
                tracker.reset();
            }
        }

        trace!("{:?} - {} until next track", *t, time_to_next_track);
        time_to_next = Some(time_to_next_track);
    }

    Ok(tracks)
}
