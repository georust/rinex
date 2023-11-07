use crate::Cli;
use statrs::statistics::Statistics;

use gnss::prelude::{Constellation, SV};
use rinex::carrier::Carrier;
use rinex::navigation::Ephemeris;
use rinex::prelude::{Observable, Rinex, RnxContext};

use rtk::{
    model::TropoComponents,
    prelude::{
        AprioriPosition, Candidate, Config, Duration, Epoch, InterpolationResult, Mode,
        PVTSolution, PseudoRange, Solver,
    },
    Vector3D,
};

use cggtts::track::SkyTracker;

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

pub fn resolve(ctx: &mut RnxContext, cli: &Cli) -> Result<HashMap<Epoch, PVTSolution>, Error> {
    // cli customizations
    let trk_duration = cli.tracking_duration();

    // parse custom config, if any
    let cfg = match cli.config() {
        Some(cfg) => cfg,
        None => Config::default(Mode::SPP),
    };

    let rtk_mode = match cli.spp() {
        true => {
            info!("spp position solver");
            Mode::SPP
        },
        false => Mode::SPP, //TODO
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
                if let Some(vec3d) = sp3.sv_position_interpolate(sv, t, order) {
                    let (elev, azi) = Ephemeris::elevation_azimuth(vec3d, apriori_ecef_wgs84);
                    Some(InterpolationResult {
                        sky_pos: vec3d.into(),
                        elevation: Some(elev),
                        azimuth: Some(azi),
                    })
                } else {
                    // debug!("{:?} ({}): sp3 interpolation failed", t, sv);
                    if let Some(vec3d) = nav_data.sv_position_interpolate(sv, t, order) {
                        let (elev, azi) = Ephemeris::elevation_azimuth(vec3d, apriori_ecef_wgs84);
                        Some(InterpolationResult {
                            sky_pos: vec3d.into(),
                            elevation: Some(elev),
                            azimuth: Some(azi),
                        })
                    } else {
                        // debug!("{:?} ({}): nav interpolation failed", t, sv);
                        None
                    }
                }
            } else {
                if let Some(vec3d) = nav_data.sv_position_interpolate(sv, t, order) {
                    let (elev, azi) = Ephemeris::elevation_azimuth(vec3d, apriori_ecef_wgs84);
                    Some(InterpolationResult {
                        sky_pos: vec3d.into(),
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

    // PVT solutions
    let mut ret: HashMap<Epoch, PVTSolution> = HashMap::new();
    // possibly provided resolved T components (contained in RINEX)
    let mut provided_clk: HashMap<Epoch, f64> = HashMap::new();

    // CGGTTS specifics
    let mut t0 = Option::<Epoch>::None;
    let mut tracker = SkyTracker::default().tracking_duration(trk_duration);

    let mut time_to_next = Option::<Duration>::None;

    let trk_nb_avg =
        (trk_duration.total_nanoseconds() / dominant_sample_rate.total_nanoseconds()) as u32;

    // print more infos
    info!("using tracking duration: {}", trk_duration);

    for (index, ((t, flag), (clk, vehicles))) in obs_data.observation().enumerate() {
        /*
         * store possibly provided clk state estimator,
         * so we can compare ours to this one later
         */
        if flag.is_ok() {
            if let Some(clk) = clk {
                provided_clk.insert(*t, *clk);
            }
        }

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
                //TODO: we only latch "L1" signal at the moment
                if observable.to_string().starts_with("L1") {
                    if observable.is_pseudorange_observable() {
                        tracker.latch_data(*t, *sv, data.obs);
                    }
                }
            }
        }

        let time_to_next_track = tracker.time_to_next_track(*t);

        if let Some(prev_time_to_next) = time_to_next {
            if time_to_next_track > prev_time_to_next {
                // time to release this track
                debug!("{:?} - releasing new track", *t);

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
                                "{:?} - {} not continuously tracked -> will not contribute",
                                *t, sv
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
                    }
                }

                match solver.resolve(*t, candidates, tropo_components, PVTSolutionType::TimeOnly) {}

                // RESET the tracker
                tracker.reset();
            }
        }

        trace!("{:?} - {} until next track", *t, time_to_next_track);
        time_to_next = Some(time_to_next_track);
        //let tropo_components = tropo_components(meteo_data, *t, lat_ddeg);

        //match solver.run(*t, candidates, tropo_components) {
        //    Ok((t, estimate)) => {
        //        debug!("{:?} : {:?}", t, estimate);
        //        ret.insert(t, estimate);
        //    },
        //    Err(e) => warn!("{:?} : {}", t, e),
        //}
    }
    Ok(ret)
}
