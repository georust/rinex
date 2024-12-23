//! CGGTTS special resolution opmoode.
use clap::ArgMatches;

use std::{cell::RefCell, collections::HashMap};

mod post_process;
pub use post_process::post_process;

mod report;
pub use report::Report;

use gnss::prelude::{Constellation, SV};

use rinex::{carrier::Carrier, prelude::Observable};

use gnss_rtk::prelude::{
    Candidate, Duration, Epoch, IonoComponents, IonosphereBias, Method, Observation, OrbitSource,
    Solver, TropoComponents, SPEED_OF_LIGHT_M_S,
};

use cggtts::{
    prelude::{CommonViewClass, Track},
    track::{FitData, GlonassChannel, SVTracker, Scheduler},
};

use crate::{
    cli::Context,
    positioning::{
        bd_model,
        cast_rtk_carrier,
        kb_model,
        ng_model, //tropo_components,
        rtk_reference_carrier,
        ClockStateProvider,
        EphemerisSource,
        Error as PositioningError,
    },
};

// fn reset_sv_tracker(sv: SV, trackers: &mut HashMap<(SV, Observable), SVTracker>) {
//     for ((k_sv, _), tracker) in trackers {
//         if *k_sv == sv {
//             tracker.reset();
//         }
//     }
// }

//TODO: see TODO down below
// fn reset_sv_sig_tracker(
//     sv_sig: (SV, Observable),
//     trackers: &mut HashMap<(SV, Observable), SVTracker>,
// ) {
//     for (k, tracker) in trackers {
//         if k == &sv_sig {
//             tracker.reset();
//         }
//     }
// }

/*
 * Resolves CGGTTS tracks from input context
 */
pub fn resolve<'a, 'b, CK: ClockStateProvider, O: OrbitSource>(
    ctx: &Context,
    eph: &'a RefCell<EphemerisSource<'b>>,
    mut clock: CK,
    mut solver: Solver<O>,
    // rx_lat_ddeg: f64,
    matches: &ArgMatches,
) -> Result<Vec<Track>, PositioningError> {
    // custom tracking duration
    let trk_duration = match matches.get_one::<Duration>("tracking") {
        Some(tracking) => {
            info!("Using custom tracking duration {:?}", *tracking);
            *tracking
        },
        _ => {
            let tracking = Duration::from_seconds(Scheduler::BIPM_TRACKING_DURATION_SECONDS.into());
            info!("Using default tracking duration {:?}", tracking);
            tracking
        },
    };

    // infaillible, at this point
    let obs_data = ctx.data.observation().unwrap();
    let nav_data = ctx.data.brdc_navigation().unwrap();
    // let meteo_data = ctx.data.meteo(); //TODO

    let dominant_sampling_period = obs_data
        .dominant_sample_rate()
        .expect("RNX2CGGTTS requires steady GNSS observations");

    // CGGTTS specifics
    let mut tracks = Vec::<Track>::new();
    let sched = Scheduler::new(trk_duration);
    let mut next_release = Option::<Epoch>::None;
    let mut trk_midpoint = Option::<Epoch>::None;
    let mut trackers = HashMap::<(SV, Observable), SVTracker>::new();

    for ((t, flag), (_clk, vehicles)) in obs_data.observation() {
        /*
         * We only consider _valid_ epochs"
         * TODO: make use of LLI marker here
         */
        if !flag.is_ok() {
            continue;
        }

        // Nearest TROPO: TODO
        // let zwd_zdd = tropo_components(meteo_data, *t, rx_lat_ddeg);

        for (sv, rinex_obs) in vehicles {
            // tries to form a candidate for each signal
            for (observable, data) in rinex_obs {
                let carrier = Carrier::from_observable(sv.constellation, observable);
                if carrier.is_err() {
                    continue; //can't proceed further
                }

                let carrier = carrier.unwrap();
                let rtk_carrier = cast_rtk_carrier(carrier);

                // We consider a reference Pseudo Range
                // and possibly gather other signals later on
                if !observable.is_pseudorange_observable() {
                    continue;
                }

                let mut ref_observable = observable.to_string();

                let mut observations = vec![Observation {
                    carrier: rtk_carrier,
                    pseudo: Some(data.obs),
                    ambiguity: None,
                    doppler: None,
                    phase: None,
                    snr: data.snr.map(|snr| snr.into()),
                }];

                // Subsidary Pseudo Range (if needed)
                if matches!(solver.cfg.method, Method::CPP | Method::PPP) {
                    // add any other signal
                    for (second_obs, second_data) in rinex_obs {
                        if second_obs == observable {
                            continue;
                        }

                        let rhs_carrier = Carrier::from_observable(sv.constellation, second_obs);

                        if rhs_carrier.is_err() {
                            continue;
                        }

                        let rhs_carrier = rhs_carrier.unwrap();
                        let rhs_rtk_carrier = cast_rtk_carrier(rhs_carrier);

                        if second_obs.is_pseudorange_observable() {
                            observations.push(Observation {
                                carrier: rhs_rtk_carrier,
                                doppler: None,
                                phase: None,
                                ambiguity: None,
                                pseudo: Some(second_data.obs),
                                snr: data.snr.map(|snr| snr.into()),
                            });
                        } else if second_obs.is_phase_observable() {
                            let lambda = rhs_carrier.wavelength();
                            if let Some(obs) = observations
                                .iter_mut()
                                .filter(|ob| ob.carrier == rhs_rtk_carrier)
                                .reduce(|k, _| k)
                            {
                                obs.phase = Some(second_data.obs * lambda);
                            } else {
                                observations.push(Observation {
                                    carrier: rhs_rtk_carrier,
                                    doppler: None,
                                    pseudo: None,
                                    ambiguity: None,
                                    phase: Some(second_data.obs * lambda),
                                    snr: data.snr.map(|snr| snr.into()),
                                });
                            }
                        } else if second_obs.is_doppler_observable() {
                            if let Some(obs) = observations
                                .iter_mut()
                                .filter(|ob| ob.carrier == rhs_rtk_carrier)
                                .reduce(|k, _| k)
                            {
                                obs.doppler = Some(second_data.obs);
                            } else {
                                observations.push(Observation {
                                    phase: None,
                                    carrier: rhs_rtk_carrier,
                                    pseudo: None,
                                    ambiguity: None,
                                    doppler: Some(second_data.obs),
                                    snr: data.snr.map(|snr| snr.into()),
                                });
                            }
                        }

                        // update ref. observable if this one is to serve as reference
                        if rtk_reference_carrier(rtk_carrier) {
                            ref_observable = second_obs.to_string();
                        }
                    }
                }

                let mut candidate = Candidate::new(*sv, *t, observations);

                // customizations
                match clock.next_clock_at(*t, *sv) {
                    Some(corr) => {
                        candidate.set_clock_correction(corr);
                    },
                    None => {
                        error!("{} ({}) - no clock correction available", *t, *sv);
                    },
                }

                if let Some((_, _, eph)) = eph.borrow_mut().select(*t, *sv) {
                    if let Some(tgd) = eph.tgd() {
                        debug!("{} ({}) - tgd: {}", *t, *sv, tgd);
                        candidate.set_group_delay(tgd);
                    }
                }

                // TODO: Meteo
                let tropo = TropoComponents::Unknown;
                candidate.set_tropo_components(tropo);

                let iono = if let Some(model) = kb_model(nav_data, *t) {
                    IonoComponents::KbModel(model)
                } else if let Some(model) = bd_model(nav_data, *t) {
                    IonoComponents::BdModel(model)
                } else if let Some(model) = ng_model(nav_data, *t) {
                    IonoComponents::NgModel(model)
                } else {
                    // TODO STEC/IONEX
                    IonoComponents::Unknown
                };

                candidate.set_iono_components(iono);

                // TODO: RTK
                //candidate.set_remote_observations(remote);

                match solver.resolve(*t, &vec![candidate]) {
                    Ok((t, pvt_solution)) => {
                        let pvt_data = pvt_solution.sv.get(sv).unwrap(); // infaillible

                        let azimuth = pvt_data.azimuth;
                        let elevation = pvt_data.elevation;

                        let refsys = pvt_solution.dt.to_seconds();
                        let correction = pvt_data.clock_correction.unwrap_or_default();
                        let refsv = refsys + correction.to_seconds();

                        /*
                         * TROPO : always present
                         *         convert to time delay (CGGTTS)
                         */
                        let mdtr = pvt_data.tropo_bias.unwrap_or_default() / SPEED_OF_LIGHT_M_S;

                        let mdio = match pvt_data.iono_bias {
                            Some(IonosphereBias::Modeled(bias)) => Some(bias),
                            _ => None,
                        };
                        let msio = match pvt_data.iono_bias {
                            Some(IonosphereBias::Measured(bias)) => Some(bias),
                            _ => None,
                        };
                        debug!(
                            "{:?} : new {}:{} solution (elev={:.2}°, azi={:.2}°, refsv={:.3E}, refsys={:.3E})",
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

                        // // verify buffer continuity
                        // if !tracker.no_gaps(dominant_sampling_period) {
                        //     // on any discontinuity we need to reset
                        //     // that tracker. This will abort the ongoing track.
                        //     tracker.reset();
                        //     warn!("{:?} - discarding {} track due to data gaps", t, sv);

                        //     // push new measurement
                        //     tracker.latch_measurement(t, fitdata);
                        //     continue; // abort for this SV
                        // }

                        if next_release.is_some() {
                            let next_release = next_release.unwrap();
                            let trk_midpoint = trk_midpoint.unwrap();

                            if t > next_release {
                                /* time to release a track */
                                let ioe = 0; //TODO

                                match tracker.fit(
                                    ioe,
                                    trk_duration,
                                    dominant_sampling_period,
                                    trk_midpoint,
                                ) {
                                    Ok(((trk_elev, trk_azi), trk_data, _iono_data)) => {
                                        info!(
                                            "{:?} : new {} cggtts solution (elev={:.2}°, azi={:.2}°, refsv={:.3E}, refsys={:.3E})",
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
                                                    match solver.cfg.method {
                                                        Method::CPP | Method::PPP => {
                                                            // TODO: grab IONOD from PVTSol
                                                            None
                                                        },
                                                        _ => None,
                                                    },
                                                    0, // TODO "rcvr_channel" > 0 if known
                                                    GlonassChannel::default(), //TODO
                                                    &ref_observable,
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
                                                    match solver.cfg.method {
                                                        Method::CPP | Method::PPP => {
                                                            // TODO: grab IONOD from PVTSol
                                                            None
                                                        },
                                                        _ => None,
                                                    },
                                                    0, // TODO "rcvr_channel" > 0 if known
                                                    &ref_observable,
                                                )
                                            },
                                        }; // match constellation
                                        tracks.push(track);
                                    },
                                    Err(e) => {
                                        warn!("{} - track fitting error: \"{}\"", t, e);
                                        // TODO: most likely we should reset the SV signal tracker here
                                    },
                                } //.fit()

                                // reset so we start a new track
                                tracker.reset();
                                // latch first measurement
                                tracker.latch_measurement(t, fitdata);
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
                        /*
                         * Any PVT resolution failures would introduce a data gap
                         * which is incompatible with CGGTTS track fitting
                         */
                        error!("pvt solver error - {}", e);
                        // if let Some(tracker) = trackers.get_mut(&(*sv, observable.clone())) {
                        //     tracker.reset();
                        // }
                    },
                } //.pvt resolve
            } // for all OBS
        } //.sv()
        next_release = Some(sched.next_track_start(*t));
        trk_midpoint = Some(next_release.unwrap() - trk_duration / 2);
        info!("{:?} - {} until next track", t, next_release.unwrap() - *t);
    } //.observations()

    tracks.sort_by(|a, b| a.epoch.cmp(&b.epoch));

    Ok(tracks)
}
