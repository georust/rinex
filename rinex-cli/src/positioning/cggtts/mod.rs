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

use hifitime::Unit;

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
    let cv_duration = match matches.get_one::<Duration>("tracking") {
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

    let mut trk_duration = cv_duration;

    if trk_duration == Scheduler::bipm_tracking_duration() {
        trk_duration = Scheduler::bipm_tracking_duration() - 3.0 * Unit::Minute;
    }

    // grab relevant structures: infaillible, at this point
    let obs_data = ctx.data.observation().unwrap();
    let nav_data = ctx.data.brdc_navigation().unwrap();

    let dominant_sampling_period = obs_data
        .dominant_sampling_interval()
        .expect("RNX2CGGTTS requires steady GNSS observations");

    let mut past_t = Option::<Epoch>::None;

    // CGGTTS specifics
    let mut tracks = Vec::<Track>::new();

    let track_scheduler = Scheduler::new(cv_duration);

    let mut trk_midpoint = Option::<Epoch>::None;
    let mut next_cv_start_time = Option::<Epoch>::None;
    let mut next_tracking_start_time = Option::<Epoch>::None;

    let mut should_skip = false;
    let mut should_release = false;

    let mut sv_observations = HashMap::<SV, Vec<Observation>>::new();

    for (t, signal) in obs_data.signal_observations_sampling_ok_iter() {
        if let Some(past_t) = past_t {
            if t != past_t && !should_skip {
                // New epoch: solving attempt
                // Creates and forwards Candidate for each SV
                for (sv, observations) in sv_observations.iter() {
                    let mut cd = Candidate::new(*sv, past_t, observations.clone());

                    // candidate "fixup" or customizations
                    match clock.next_clock_at(past_t, *sv) {
                        Some(dt) => cd.set_clock_correction(dt),
                        None => error!("{} ({}) - no clock correction available", past_t, *sv),
                    }

                    if let Some((_, _, eph)) = eph.borrow_mut().select(past_t, *sv) {
                        if let Some(tgd) = eph.tgd() {
                            debug!("{} ({}) - tgd: {}", past_t, *sv, tgd);
                            cd.set_group_delay(tgd);
                        }
                    }

                    let tropo = TropoComponents::Unknown;
                    cd.set_tropo_components(tropo);

                    let mut iono = IonoComponents::Unknown;

                    if let Some(model) = kb_model(nav_data, past_t) {
                        iono = IonoComponents::KbModel(model);
                    } else if let Some(model) = ng_model(nav_data, past_t) {
                        iono = IonoComponents::NgModel(model);
                    } else if let Some(model) = bd_model(nav_data, past_t) {
                        iono = IonoComponents::BdModel(model);
                    }

                    match iono {
                        IonoComponents::Unknown => {
                            warn!("{} ({}) - undefined ionosphere parameters", past_t, *sv)
                        },
                        IonoComponents::KbModel(_) => info!(
                            "{} ({}) - using KLOBUCHAR ionosphere parameters",
                            past_t, *sv
                        ),
                        IonoComponents::NgModel(_) => info!(
                            "{} ({}) - using NEQUICK-G ionosphere parameters",
                            past_t, *sv
                        ),
                        IonoComponents::BdModel(_) => {
                            info!("{} ({}) - using BDGIM ionosphere parameters", past_t, *sv)
                        },
                        _ => {},
                    }

                    cd.set_iono_components(iono);

                    match solver.resolve(past_t, &[cd]) {
                        Ok((t, pvt_solution)) => {
                            info!(
                                "({:?} ({}) : new pvt solution {:?}",
                                past_t, signal.sv, pvt_solution
                            );
                        },
                        Err(e) => {
                            // any PVT solution failure will introduce a gap in the track fitter
                            error!("pvt solver error: {}", e);
                        },
                    }
                } // for each sv

                sv_observations.clear();
            } // new track attempt
        } // epoch

        let carrier = Carrier::from_observable(signal.sv.constellation, &signal.observable);
        if carrier.is_err() {
            continue;
        }

        let carrier = carrier.unwrap();
        let rtk_carrier = cast_rtk_carrier(carrier);

        if let Some((_, observations)) = sv_observations
            .iter_mut()
            .filter(|(k, _)| **k == signal.sv)
            .reduce(|k, _| k)
        {
            if let Some(observation) = observations
                .iter_mut()
                .filter(|k| k.carrier == rtk_carrier)
                .reduce(|k, _| k)
            {
                match signal.observable {
                    Observable::PhaseRange(_) => {
                        observation.set_ambiguous_phase_range(signal.value);
                    },
                    Observable::PseudoRange(_) => {
                        observation.set_pseudo_range(signal.value);
                    },
                    Observable::Doppler(_) => {
                        observation.set_doppler(signal.value);
                    },
                    _ => {},
                }
            } else {
            }
        } else {
            match signal.observable {
                Observable::PhaseRange(_) => {
                    sv_observations.insert(
                        signal.sv,
                        vec![Observation::ambiguous_phase_range(
                            rtk_carrier,
                            signal.value,
                            None,
                        )],
                    );
                },
                Observable::PseudoRange(_) => {
                    sv_observations.insert(
                        signal.sv,
                        vec![Observation::pseudo_range(rtk_carrier, signal.value, None)],
                    );
                },
                Observable::Doppler(_) => {
                    sv_observations.insert(
                        signal.sv,
                        vec![Observation::doppler(rtk_carrier, signal.value, None)],
                    );
                },
                _ => {},
            }
        }

        // When last Epoch of the CV period is reached: we should release a track
        let next_release_duration = next_cv_start_time.unwrap() - t;

        should_release = next_release_duration <= dominant_sampling_period;
        should_release &= next_release_duration > Duration::ZERO;

        trk_midpoint = Some(next_cv_start_time.unwrap() - trk_duration / 2);

        info!(
            "{:?} - {} until next cv period",
            t,
            next_cv_start_time.unwrap() - t,
        );

        // skip the prepare duration
        should_skip = next_release_duration < Duration::ZERO;

        past_t = Some(t);
    }

    //         for (sv, rinex_obs) in vehicles {
    //                 let target = &(*sv, observable.clone());

    //                 let tracker = match trackers.get_mut(target) {
    //                     None => {
    //                         // initialize new tracker
    //                         trackers.insert((*sv, observable.clone()), SVTracker::default());
    //                         trackers.get_mut(target).unwrap()
    //                     },
    //                     Some(tracker) => tracker,
    //                 };

    //                 match solver.resolve(*t, &vec![candidate]) {
    //                     Ok((t, pvt_solution)) => {
    //                         let pvt_data = pvt_solution.sv.get(sv).unwrap(); // infaillible

    //                         let azimuth = pvt_data.azimuth;
    //                         let elevation = pvt_data.elevation;

    //                         let refsys = pvt_solution.dt.to_seconds();
    //                         let correction = pvt_data.clock_correction.unwrap_or_default();
    //                         let refsv = refsys + correction.to_seconds();

    //                         /*
    //                          * TROPO : always present
    //                          *         convert to time delay (CGGTTS)
    //                          */
    //                         let mdtr = pvt_data.tropo_bias.unwrap_or_default() / SPEED_OF_LIGHT_M_S;

    //                         let mdio = match pvt_data.iono_bias {
    //                             Some(IonosphereBias::Modeled(bias)) => Some(bias),
    //                             _ => None,
    //                         };
    //                         let msio = match pvt_data.iono_bias {
    //                             Some(IonosphereBias::Measured(bias)) => Some(bias),
    //                             _ => None,
    //                         };
    //                         debug!(
    //                         "{:?} : new {}:{} solution (elev={:.2}°, azi={:.2}°, refsv={:.3E}, refsys={:.3E})",
    //                         t, sv, observable, elevation, azimuth, refsv, refsys
    //                     );

    //                         let fitdata = FitData {
    //                             refsv,
    //                             refsys,
    //                             mdtr,
    //                             mdio,
    //                             msio,
    //                             azimuth,
    //                             elevation,
    //                         };

    //                         // // verify buffer continuity
    //                         // if !tracker.no_gaps(dominant_sampling_period) {
    //                         //     // on any discontinuity we need to reset
    //                         //     // that tracker. This will abort the ongoing track.
    //                         //     tracker.reset();
    //                         //     warn!("{:?} - discarding {} track due to data gaps", t, sv);

    //                         //     // push new measurement
    //                         //     tracker.latch_measurement(t, fitdata);
    //                         //     continue; // abort for this SV
    //                         // }

    //                         if next_tracking_start_time.is_some() {
    //                             let next_tracking_start_time = next_tracking_start_time.unwrap();
    //                             let trk_midpoint = trk_midpoint.unwrap();

    //                             if should_release {
    //                                 /* time to release a track */
    //                                 let ioe = 0; //TODO
    //                                              // latch last measurement
    //                                 tracker.latch_measurement(t, fitdata);

    //                                 match tracker.fit(
    //                                     ioe,
    //                                     trk_duration,
    //                                     dominant_sampling_period,
    //                                     trk_midpoint,
    //                                 ) {
    //                                     Ok(((trk_elev, trk_azi), trk_data, _iono_data)) => {
    //                                         info!(
    //                                         "{:?} : new {} cggtts solution (elev={:.2}°, azi={:.2}°, refsv={:.3E}, refsys={:.3E})",
    //                                         t,
    //                                         sv,
    //                                         trk_elev,
    //                                         trk_azi,
    //                                         trk_data.refsv,
    //                                         trk_data.refsys
    //                                     );

    //                                         let track = match sv.constellation {
    //                                             Constellation::Glonass => {
    //                                                 Track::new_glonass(
    //                                                     *sv,
    //                                                     next_tracking_start_time,
    //                                                     trk_duration,
    //                                                     CommonViewClass::SingleChannel,
    //                                                     trk_elev,
    //                                                     trk_azi,
    //                                                     trk_data,
    //                                                     match solver.cfg.method {
    //                                                         Method::CPP | Method::PPP => {
    //                                                             // TODO: grab IONOD from PVTSol
    //                                                             None
    //                                                         },
    //                                                         _ => None,
    //                                                     },
    //                                                     0, // TODO "rcvr_channel" > 0 if known
    //                                                     GlonassChannel::default(), //TODO
    //                                                     &ref_observable,
    //                                                 )
    //                                             },
    //                                             _ => {
    //                                                 Track::new(
    //                                                     *sv,
    //                                                     next_tracking_start_time,
    //                                                     trk_duration,
    //                                                     CommonViewClass::SingleChannel,
    //                                                     trk_elev,
    //                                                     trk_azi,
    //                                                     trk_data,
    //                                                     match solver.cfg.method {
    //                                                         Method::CPP | Method::PPP => {
    //                                                             // TODO: grab IONOD from PVTSol
    //                                                             None
    //                                                         },
    //                                                         _ => None,
    //                                                     },
    //                                                     0, // TODO "rcvr_channel" > 0 if known
    //                                                     &ref_observable,
    //                                                 )
    //                                             },
    //                                         }; // match constellation
    //                                         tracks.push(track);
    //                                     },
    //                                     Err(e) => {
    //                                         warn!("{} - track fitting error: \"{}\"", t, e);
    //                                         // TODO: most likely we should reset the SV signal tracker here
    //                                     },
    //                                 } //.fit()
    //                             }
    //                             // time to release a track
    //                             else {
    //                                 tracker.latch_measurement(t, fitdata);
    //                             }
    //                         }
    //                         //release.is_none()
    //                         else {
    //                             tracker.latch_measurement(t, fitdata);
    //                         }
    //                     },
    //                 } //.pvt resolve
    //                   // after release, reset so we start a new track
    //                 if should_release {
    //                     tracker.reset();
    //                 }
    //             } // for all OBS
    //         } //.sv()
    //     }
    //     next_tracking_start_time = Some(sched.next_track_start(*t));
    //     next_cv_start_time = next_tracking_start_time;
    //     if cv_duration == Scheduler::bipm_tracking_duration() {
    //         let cv_prepare_duration = 3.0 * Unit::Minute;
    //         next_cv_start_time = Some(next_cv_start_time.unwrap() - cv_prepare_duration);
    //     }

    tracks.sort_by(|a, b| a.epoch.cmp(&b.epoch));

    Ok(tracks)
}
