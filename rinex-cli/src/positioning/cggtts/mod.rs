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
    Candidate, Carrier as RTKCarrier, Duration, Epoch, IonoComponents, IonosphereBias, Method,
    Observation, OrbitSource, Solver, TropoComponents, SPEED_OF_LIGHT_M_S,
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

/// Resolves CGGTTS tracks from input context
pub fn resolve<'a, 'b, CK: ClockStateProvider, O: OrbitSource>(
    ctx: &Context,
    eph: &'a RefCell<EphemerisSource<'b>>,
    mut clock: CK,
    mut solver: Solver<O>,
    method: Method,
    matches: &ArgMatches,
) -> Result<Vec<Track>, PositioningError> {
    // possible custom tracking
    let (cv_period_duration, cv_warmup_duration) = match matches.get_one::<Duration>("tracking") {
        Some(tracking) => {
            info!("Using custom tracking duration {:?}", *tracking);
            (*tracking, Duration::ZERO)
        },
        _ => {
            let tracking = Duration::from_seconds(Scheduler::BIPM_TRACKING_DURATION_SECONDS.into());
            info!("Using default tracking duration {:?}", tracking);
            (tracking, 3.0 * Unit::Minute)
        },
    };

    let fit_duration = cv_period_duration - cv_warmup_duration;
    let half_fit_duration = fit_duration / 2.0;

    let obs_data = ctx
        .data
        .observation()
        .expect("RNX2CGGTTS requires OBS RINEX");

    let nav_data = ctx
        .data
        .brdc_navigation()
        .expect("RNX2CGGTTS requires NAV RINEX");

    let t0 = obs_data
        .first_epoch()
        .expect("failed to determine first epoch, empty observations?");

    let dominant_sampling_period = obs_data
        .dominant_sampling_interval()
        .expect("RNX2CGGTTS requires steady GNSS observations");

    let mut past_t = t0;
    let mut tracks = Vec::<Track>::new();
    let mut sv_observations = HashMap::<SV, Vec<Observation>>::new();

    let track_scheduler = Scheduler::new(cv_period_duration);

    let mut cv_period_start = track_scheduler.next_track_start(t0);
    let mut trk_midpoint = cv_period_start + cv_warmup_duration + half_fit_duration;

    // we have one reference signal per SV
    let mut sv_reference = HashMap::<SV, Observable>::new();

    // we have one SV tracker per SV and reference signal
    let mut sv_trackers = HashMap::<(SV, Observable), SVTracker>::new();

    for (index, (t, signal)) in obs_data.signal_observations_sampling_ok_iter().enumerate() {
        // PVT solution contribution to fitting algorithm
        let contributes = past_t >= cv_period_start + cv_warmup_duration;

        // Time remaining, in case we're in warmup interval
        let remaining_warmup = cv_period_start + cv_warmup_duration - past_t;

        // Time before next publication (track fitting attempt).
        // "now" is past the last contributing epoch
        let should_release = past_t > (cv_period_start + cv_warmup_duration) + cv_period_duration;

        if index > 0 && t != past_t {
            // time to next CGGTTS publication
            let dt = cv_period_start + cv_warmup_duration + cv_period_duration - t;
            info!("{:?} - {} until next CGGTTS publication", past_t, dt);

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
                        // grab "reference" signal
                        let sv_reference_obs = match sv_reference.get(&sv) {
                            Some(obs) => obs.clone(),
                            None => panic!("cggtts: no reference signal for {}", sv),
                        };

                        // grab SV tracker or initialize
                        if sv_trackers.get(&(*sv, sv_reference_obs.clone())).is_none() {
                            sv_trackers
                                .insert((*sv, sv_reference_obs.clone()), SVTracker::default());
                        }

                        let mut sv_tracker = sv_trackers
                            .get_mut(&(*sv, sv_reference_obs.clone()))
                            .unwrap();

                        let pvt_data = pvt_solution.sv.get(sv).unwrap(); // infaillible

                        let (azimuth, elevation) = (pvt_data.azimuth, pvt_data.elevation);

                        let refsys = pvt_solution.dt.to_seconds();
                        let correction = pvt_data.clock_correction.unwrap_or_default();
                        let refsv = refsys + correction.to_seconds();

                        info!(
                            "({:?} ({}) : new pvt solution (elev={:.2}°, azim={:.2}°, refsv={:.3E}, refsys={:.3})",
                            past_t, signal.sv, elevation, azimuth, refsv, refsys,
                        );

                        if !contributes {
                            debug!(
                                "{:?} ({}) - warmup (still {} to go)",
                                past_t, sv, remaining_warmup
                            );
                        } else {
                            // PVT solution contributes to the CGGTTS fitting algorithm
                            debug!("{:?} ({}) - contributes", past_t, sv);

                            // tropod model
                            let mdtr = pvt_data.tropo_bias.unwrap_or_default() / SPEED_OF_LIGHT_M_S;

                            // ionod
                            let mdio = match pvt_data.iono_bias {
                                Some(IonosphereBias::Modeled(bias)) => Some(bias),
                                _ => None,
                            };

                            let msio = match pvt_data.iono_bias {
                                Some(IonosphereBias::Measured(bias)) => Some(bias),
                                _ => None,
                            };

                            // track fitting
                            let fitdata = FitData {
                                refsv,
                                refsys,
                                mdtr,
                                mdio,
                                msio,
                                azimuth,
                                elevation,
                            };

                            sv_tracker.latch_measurement(t, fitdata);

                            let ioe = 0; // TODO

                            if should_release {
                                match sv_tracker.fit(
                                    ioe,
                                    fit_duration,
                                    dominant_sampling_period,
                                    trk_midpoint,
                                ) {
                                    Ok(((trk_elev, trk_azim), trk_data, _iono_data)) => {
                                        info!("{:?} ({}) - new cggtts solution (elev={:.2}°, azim={:.2}°, refsv={:.3E}, refsys={:.3E})",
                                            past_t,
                                            sv,
                                            trk_elev,
                                            trk_azim,
                                            trk_data.refsv,
                                            trk_data.refsys,
                                        );
                                    },
                                    Err(e) => {
                                        error!("{:?} - track fitting error: {}", past_t, e);
                                    },
                                } //tracker.fit()
                            } // should release
                        } //contributes
                    },
                    Err(e) => {
                        // any PVT solution failure will introduce a gap in the track fitter
                        error!("pvt solver error: {}", e);
                    },
                }
            } // for each sv

            sv_observations.clear();

            if should_release {
                // we did (or at least attempted to) publish a track

                // reset all trackers
                for (_, sv_tracker) in sv_trackers.iter_mut() {
                    sv_tracker.reset();
                }

                // define new period
                cv_period_start = track_scheduler.next_track_start(past_t);
                trk_midpoint = cv_period_start + cv_warmup_duration + half_fit_duration;

                debug!(
                    "{:?} - new cv period: start={:?} midpoint={:?}",
                    past_t, cv_period_start, trk_midpoint
                );
            }
        } // new epoch

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
                match signal.observable {
                    Observable::PhaseRange(_) => {
                        observations.push(Observation::ambiguous_phase_range(
                            rtk_carrier,
                            signal.value,
                            None,
                        ));
                    },
                    Observable::PseudoRange(_) => {
                        observations.push(Observation::pseudo_range(
                            rtk_carrier,
                            signal.value,
                            None,
                        ));
                    },
                    Observable::Doppler(_) => {
                        observations.push(Observation::doppler(rtk_carrier, signal.value, None));
                    },
                    _ => {},
                }
            }
        } else {
            // first SV encounter
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

                    if method == Method::PPP {
                        if matches!(
                            rtk_carrier,
                            RTKCarrier::L1 | RTKCarrier::E1 | RTKCarrier::B1aB1c | RTKCarrier::B1I
                        ) {
                            sv_reference.insert(signal.sv, signal.observable.clone());
                        }
                    }
                },
                Observable::PseudoRange(_) => {
                    sv_observations.insert(
                        signal.sv,
                        vec![Observation::pseudo_range(rtk_carrier, signal.value, None)],
                    );

                    if method != Method::PPP {
                        if matches!(
                            rtk_carrier,
                            RTKCarrier::L1 | RTKCarrier::E1 | RTKCarrier::B1aB1c | RTKCarrier::B1I
                        ) {
                            sv_reference.insert(signal.sv, signal.observable.clone());
                        }
                    }
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
        past_t = t;
    }

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

    Ok(tracks)
}
