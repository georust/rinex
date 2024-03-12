//! CGGTTS special resolution opmoode.
use clap::ArgMatches;
use std::collections::HashMap;
use std::str::FromStr;

mod post_process;
pub use post_process::{post_process, Error as PostProcessingError};

use gnss::prelude::{Constellation, SV};

use rinex::{carrier::Carrier, navigation::Ephemeris, prelude::Observable};

use rtk::prelude::{
    Candidate,
    Duration,
    Epoch,
    InterpolationResult,
    IonosphericBias,
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

use crate::cli::Context;
use crate::positioning::{
    bd_model, kb_model, ng_model, tropo_components, Error as PositioningError,
};

fn reset_sv_tracker(sv: SV, trackers: &mut HashMap<(SV, Observable), SVTracker>) {
    for ((k_sv, _), tracker) in trackers {
        if *k_sv == sv {
            tracker.reset();
        }
    }
}

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
pub fn resolve<APC, I>(
    ctx: &Context,
    mut solver: Solver<APC, I>,
    rx_lat_ddeg: f64,
    matches: &ArgMatches,
) -> Result<Vec<Track>, PositioningError>
where
    APC: Fn(Epoch, SV, f64) -> Option<(f64, f64, f64)>,
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
    let meteo_data = ctx.data.meteo();

    let clk_data = ctx.data.clock();
    let _has_clk_data = clk_data.is_some();

    let _sp3_data = ctx.data.sp3();
    let sp3_has_clock = ctx.data.sp3_has_clock();

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
         * we only consider "OK" Epochs
         */
        if !flag.is_ok() {
            continue;
        }

        // Nearest TROPO
        let zwd_zdd = tropo_components(meteo_data, *t, rx_lat_ddeg);

        for (sv, observations) in vehicles {
            let sv_eph = nav_data.sv_ephemeris(*sv, *t);

            if sv_eph.is_none() {
                warn!("{:?} ({}) : undetermined ephemeris", t, sv);
                reset_sv_tracker(*sv, &mut trackers); // reset for this SV entirely
                continue; // can't proceed further
            }

            // determine TOE
            let (toe, sv_eph) = sv_eph.unwrap();
            /*
             * Clock state
             *  1. Prefer CLK product
             *  2. Prefer SP3 product
             *  3. Radio last option
             */
            let clock_state = if let Some(clk) = clk_data {
                if let Some((_, profile)) = clk.precise_sv_clock_interpolate(*t, *sv) {
                    (
                        profile.bias,
                        profile.drift.unwrap_or(0.0),
                        profile.drift_change.unwrap_or(0.0),
                    )
                } else {
                    /*
                     * interpolation failure.
                     * Do not interpolate other products: SV will not be presented.
                     */
                    continue;
                }
            } else if sp3_has_clock {
                panic!("sp3 (clock) interpolation not ready yet: prefer broadcast or clk product");
            } else {
                sv_eph.sv_clock() // BRDC case
            };

            // determine clock correction
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

                if observable.is_pseudorange_observable() {
                    code = Some(Observation {
                        frequency,
                        snr: { data.snr.map(|snr| snr.into()) },
                        value: data.obs,
                    });
                } else if observable.is_phase_observable() {
                    phase = Some(Observation {
                        frequency,
                        snr: { data.snr.map(|snr| snr.into()) },
                        value: data.obs,
                    });
                }

                // we only one phase or code here
                if code.is_none() && phase.is_none() {
                    continue;
                }

                let mut doppler = Option::<Observation>::None;
                let doppler_to_match =
                    Observable::from_str(&format!("D{}", &observable.to_string()[..1])).unwrap();

                for (observable, data) in observations {
                    if observable.is_doppler_observable() && observable == &doppler_to_match {
                        doppler = Some(Observation {
                            frequency,
                            snr: { data.snr.map(|snr| snr.into()) },
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

                        let mdio = pvt_data.iono_bias.modeled;

                        let msio = pvt_data.iono_bias.measured;

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
                                    Ok(((trk_elev, trk_azi), trk_data, _iono_data)) => {
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
                                    Err(e) => {
                                        warn!("{:?} - track fitting error: \"{}\"", t, e);
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
