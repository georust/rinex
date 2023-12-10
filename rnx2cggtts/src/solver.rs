use crate::Cli;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

use gnss::prelude::{Constellation, SV};

use rinex::{
    carrier::Carrier,
    navigation::Ephemeris,
    prelude::{Observable, Rinex, RnxContext},
};

use rtk::prelude::{
    AprioriPosition,
    BdModel,
    Candidate,
    Config,
    Duration,
    Epoch,
    Filter,
    InterpolatedPosition,
    InterpolationResult,
    IonosphericBias,
    KbModel,
    Method,
    NgModel,
    Observation,
    PVTSolutionType,
    Solver,
    TroposphericBias, //TimeScale
    Vector3,
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

fn reset_sv_tracker(sv: SV, trackers: &mut HashMap<(SV, Observable), SVTracker>) {
    for ((k_sv, _), tracker) in trackers {
        if *k_sv == sv {
            tracker.reset();
        }
    }
}

fn reset_sv_sig_tracker(
    sv_sig: (SV, Observable),
    trackers: &mut HashMap<(SV, Observable), SVTracker>,
) {
    for (k, tracker) in trackers {
        if k == &sv_sig {
            tracker.reset();
        }
    }
}

pub fn resolve(ctx: &mut RnxContext, cli: &Cli) -> Result<Vec<Track>, Error> {
    // custom tracking duration
    let trk_duration = cli.tracking_duration();
    info!("tracking duration set to {}", trk_duration);

    // parse custom config, if any
    let cfg = match cli.config() {
        Some(cfg) => cfg,
        None => Config::default(),
    };

    match cfg.method {
        Method::SPP => info!("single point positioning"),
        Method::PPP => info!("precise point positioning"),
    };

    let pos = match cli.manual_apc() {
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

    let dominant_sampling_period = obs_data
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
        &cfg,
        apriori,
        /* state vector interpolator */
        |t, sv, order| {
            /* SP3 source is prefered */
            if let Some(sp3) = sp3_data {
                if let Some((x, y, z)) = sp3.sv_position_interpolate(sv, t, order) {
                    let (x, y, z) = (x * 1.0E3, y * 1.0E3, z * 1.0E3);
                    let (elevation, azimuth) =
                        Ephemeris::elevation_azimuth((x, y, z), apriori_ecef);
                    Some(
                        InterpolationResult::from_mass_center_position((x, y, z))
                            .with_elevation_azimuth((elevation, azimuth)),
                    )
                } else {
                    // debug!("{:?} ({}): sp3 interpolation failed", t, sv);
                    if let Some((x, y, z)) = nav_data.sv_position_interpolate(sv, t, order) {
                        let (x, y, z) = (x * 1.0E3, y * 1.0E3, z * 1.0E3);
                        let (elevation, azimuth) =
                            Ephemeris::elevation_azimuth((x, y, z), apriori_ecef);
                        Some(
                            InterpolationResult::from_apc_position((x, y, z))
                                .with_elevation_azimuth((elevation, azimuth)),
                        )
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
                    Some(
                        InterpolationResult::from_apc_position((x, y, z))
                            .with_elevation_azimuth((elevation, azimuth)),
                    )
                } else {
                    // debug!("{:?} ({}): nav interpolation failed", t, sv);
                    None
                }
            }
        },
    )?;

    // CGGTTS specifics
    let mut tracks = Vec::<Track>::new();
    let sched = Scheduler::new(trk_duration);
    let mut next_release = Option::<Epoch>::None;
    let mut trk_midpoint = Option::<Epoch>::None;
    let mut trackers = HashMap::<(SV, Observable), SVTracker>::new();

    for ((t, flag), (_clk, vehicles)) in obs_data.observation() {
        /*
         * we only consider "OK" Epochs
         */
        if !flag.is_ok() {
            continue;
        }

        // Nearest TROPO
        let zwd_zdd = tropo_components(meteo_data, *t, lat_ddeg);

        for (sv, observations) in vehicles {
            let sv_eph = nav_data.sv_ephemeris(*sv, *t);

            if sv_eph.is_none() {
                warn!("{:?} ({}) : undetermined ephemeris", t, sv);
                reset_sv_tracker(*sv, &mut trackers); // reset for this SV entirely
                continue; // can't proceed further
            }

            let (toe, sv_eph) = sv_eph.unwrap();
            let clock_state = sv_eph.sv_clock();
            let clock_corr = Ephemeris::sv_clock_corr(*sv, clock_state, *t, toe);
            let clock_state = Vector3::new(clock_state.0, clock_state.1, clock_state.2);

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

            // form PVT "candidate" for each signal
            for (observable, data) in observations {
                let carrier = Carrier::from_observable(sv.constellation, observable);
                if carrier.is_err() {
                    continue; //can't proceed further
                }

                let carrier = carrier.unwrap();
                let frequency = carrier.frequency();

                let mut code = Option::<Observation>::None;
                let mut phase = Option::<Observation>::None;
                let mut assoc_doppler = Option::<Observation>::None;

                if observable.is_pseudorange_observable() {
                    code = Some(Observation {
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
                    phase = Some(Observation {
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

                if code.is_none() && phase.is_none() {
                    continue;
                }

                let doppler = Option::<Observation>::None;
                let doppler_to_match =
                    Observable::from_str(&format!("D{}", &observable.to_string()[..1])).unwrap();

                for (observable, data) in observations {
                    if observable.is_doppler_observable() && observable == &doppler_to_match {
                        assoc_doppler = Some(Observation {
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

                let candidate = match code {
                    Some(code) => {
                        let doppler = match doppler {
                            Some(doppler) => vec![doppler],
                            None => vec![],
                        };
                        Candidate::new(
                            *sv,
                            *t,
                            clock_state,
                            clock_corr,
                            vec![code],
                            vec![],
                            doppler,
                        )
                    },
                    None => {
                        let phase = phase.unwrap(); // infaillible
                        let doppler = match doppler {
                            Some(doppler) => vec![doppler],
                            None => vec![],
                        };
                        Candidate::new(
                            *sv,
                            *t,
                            clock_state,
                            clock_corr,
                            vec![],
                            vec![phase],
                            doppler,
                        )
                    },
                };

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
                    PVTSolutionType::TimeOnly,
                    vec![candidate],
                    &iono_bias,
                    &tropo_bias,
                ) {
                    Ok((t, pvt_solution)) => {
                        let pvt_data = pvt_solution.sv.get(sv).unwrap(); // infaillible

                        let azimuth = pvt_data.azimuth;
                        let elevation = pvt_data.elevation;

                        let refsys = pvt_solution.dt;
                        let refsv = pvt_solution.dt + clock_corr.to_seconds();

                        /*
                         * TROPO : always present
                         *         convert to time delay (CGGTTS)
                         */
                        let mdtr = match pvt_data.tropo_bias.value() {
                            Some(tropo) => tropo / 299792458.0,
                            None => 0.0_f64,
                        };

                        let mdio = match pvt_data.iono_bias.modeled {
                            Some(iono) => Some(iono),
                            None => None,
                        };

                        let msio = match pvt_data.iono_bias.measured {
                            Some(iono) => Some(iono),
                            None => None,
                        };

                        debug!(
                            "{:?} : new {}:{} PVT solution (elev={:.2}°, azi={:.2}°, REFSV={:.3E}, REFSYS={:.3E})",
                            t, sv, observable, elevation, azimuth, refsv, refsys
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

                        let target = &(*sv, observable.clone());

                        let tracker = match trackers.get_mut(target) {
                            None => {
                                // initialize new tracker
                                trackers.insert((*sv, observable.clone()), SVTracker::default());
                                trackers.get_mut(target).unwrap()
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
                                                    &observable.to_string(),
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
                                                    None, //TODO "iono": once L2/L5 unlocked,
                                                    0,    // TODO "rcvr_channel" > 0 if known
                                                    &observable.to_string(),
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
                        if let Some(tracker) = trackers.get_mut(&(*sv, observable.clone())) {
                            tracker.reset();
                        }
                    },
                } //.pvt resolve
            } // for all OBS
        } //.sv()
        next_release = Some(sched.next_track_start(*t));
        trk_midpoint = Some(next_release.unwrap() - trk_duration / 2);
        info!("{:?} - {} until next track", t, next_release.unwrap() - *t);
    } //.observations()

    Ok(tracks)
}
