//! CGGTTS special resolution opmoode.
use clap::ArgMatches;
use std::collections::HashMap;

mod post_process;
pub use post_process::post_process;

mod report;
pub use report::Report;

use gnss::prelude::{Constellation, SV};

use rinex::{carrier::Carrier, prelude::Observable};

use rtk::prelude::{
    Candidate,
    Duration,
    Epoch,
    InterpolationResult,
    IonosphereBias,
    Method,
    PhaseRange,
    PseudoRange,
    Solver,
    TroposphereBias, //TimeScale
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
        rtk_carrier_cast,
        rtk_reference_carrier,
        Error as PositioningError,
        Time,
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
pub fn resolve<I>(
    ctx: &Context,
    mut solver: Solver<I>,
    // rx_lat_ddeg: f64,
    matches: &ArgMatches,
) -> Result<Vec<Track>, PositioningError>
where
    I: Fn(Epoch, SV, usize) -> Option<InterpolationResult>,
{
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

    // let mut initialized = false; // solver state
    let mut time = Time::from_ctx(ctx);

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

        for (sv, observations) in vehicles {
            let sv_eph = nav_data.sv_ephemeris(*sv, *t);

            if sv_eph.is_none() {
                warn!("{:?} ({}) : undetermined ephemeris", t, sv);
                // reset_sv_tracker(*sv, &mut trackers);
                continue; // can't proceed further
            }

            // determine TOE
            let (_toe, sv_eph) = sv_eph.unwrap();
            let clock_corr = match time.next_at(*t, *sv) {
                Some(dt) => dt,
                None => {
                    error!("{} ({}) - failed to determine clock correction", *t, *sv);
                    continue;
                },
            };

            let iono_bias = IonosphereBias {
                kb_model: kb_model(nav_data, *t),
                bd_model: bd_model(nav_data, *t),
                ng_model: ng_model(nav_data, *t),
                stec_meas: None, //TODO
            };

            let tropo_bias = TroposphereBias {
                total: None,   //TODO
                zwd_zdd: None, // TODO
            };

            // tries to form a candidate for each signal
            for (observable, data) in observations {
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

                let mut codes = vec![PseudoRange {
                    carrier: rtk_carrier,
                    snr: { data.snr.map(|snr| snr.into()) },
                    value: data.obs,
                }];

                // Subsidary Pseudo Range (if needed)
                match solver.cfg.method {
                    Method::CPP | Method::PPP => {
                        // locate secondary signal
                        for (second_obs, second_data) in observations {
                            if !second_obs.is_pseudorange_observable() {
                                continue;
                            }
                            let rhs_carrier =
                                Carrier::from_observable(sv.constellation, second_obs);
                            if rhs_carrier.is_err() {
                                continue;
                            }
                            let rhs_carrier = rhs_carrier.unwrap();
                            let rtk_carrier = cast_rtk_carrier(rhs_carrier);

                            if rhs_carrier != carrier {
                                codes.push(PseudoRange {
                                    carrier: rtk_carrier,
                                    value: second_data.obs,
                                    snr: { data.snr.map(|snr| snr.into()) },
                                });
                            }
                            // update ref. observable if this one is to serve as reference
                            if rtk_reference_carrier(rtk_carrier) {
                                ref_observable = second_obs.to_string();
                            }
                        }
                    },
                    _ => {}, // not needed
                };

                // Dual Phase Range (if needed)
                //let mut doppler = Option::<Observation>::None;
                let mut phases = Vec::<PhaseRange>::with_capacity(4);

                if solver.cfg.method == Method::PPP {
                    for code in &codes {
                        let target_carrier = rtk_carrier_cast(code.carrier);
                        for (obs, data) in observations {
                            if !obs.is_phase_observable() {
                                continue;
                            }
                            let carrier = Carrier::from_observable(sv.constellation, obs);
                            if carrier.is_err() {
                                continue;
                            }
                            let carrier = carrier.unwrap();

                            if target_carrier != carrier {
                                continue;
                            }
                            phases.push(PhaseRange {
                                ambiguity: None,
                                carrier: code.carrier,
                                value: data.obs,
                                snr: { data.snr.map(|snr| snr.into()) },
                            });
                        }
                    }
                };

                let candidate = Candidate::new(*sv, *t, clock_corr, sv_eph.tgd(), codes, phases);

                match solver.resolve(*t, &vec![candidate], &iono_bias, &tropo_bias) {
                    Ok((t, pvt_solution)) => {
                        let pvt_data = pvt_solution.sv.get(sv).unwrap(); // infaillible

                        let azimuth = pvt_data.azimuth;
                        let elevation = pvt_data.elevation;

                        let refsys = pvt_solution.dt.to_seconds();
                        let refsv = refsys + clock_corr.to_seconds();

                        /*
                         * TROPO : always present
                         *         convert to time delay (CGGTTS)
                         */
                        let mdtr = match pvt_data.tropo_bias.value() {
                            Some(tropo) => tropo / 299792458.0,
                            None => 0.0_f64,
                        };

                        let mdio = pvt_data.iono_bias.modeled;
                        let msio = pvt_data.iono_bias.measured;
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
