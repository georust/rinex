//! CGGTTS special resolution opmoode.
use clap::ArgMatches;
use itertools::Itertools;

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

use super::rtk_carrier_cast;

/// CGGTTS [Track]s resolution attempt from input [Context],
/// [EphemerisSource], [ClockStateProvider] and [OrbitSource]
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
    let obs_data = ctx.data.observation_data().unwrap();
    let nav_data = ctx.data.brdc_navigation_data().unwrap();
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

    // Consume all signals
    for (key, observations) in obs_data.observations_iter() {
        if !key.flag.is_ok() {
            // Discards bad sampling conditions
            continue;
        }

        let t = key.epoch;

        // TODO: nearest tropo model (in time)
        // let zwd_zdd = tropo_components(meteo_data, *t, rx_lat_ddeg);
        let tropo = TropoComponents::Unknown;

        // Consume all SV + frequencies + pseudo range
        for (sv, carrier, pr_observable) in observations
            .signals
            .iter()
            .filter_map(|sig| {
                if sig.observable.is_pseudo_range_observable() {
                    // retrieve frequency
                    if let Ok(carrier) = sig.observable.carrier(sig.sv.constellation) {
                        Some((sig.sv, carrier, sig.observable.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unique()
            .sorted()
        {
            let rtk_carrier = cast_rtk_carrier(carrier);

            let mut rtk_observations = vec![];

            // add more observations (if we can or have to)
            for rhs_signal in observations.signals.iter() {
                if rhs_signal.sv == sv {
                    if let Ok(rhs_carrier) =
                        rhs_signal.observable.carrier(rhs_signal.sv.constellation)
                    {
                        let rtk_carrier = cast_rtk_carrier(carrier);

                        if rhs_carrier == carrier {
                            // insert pseudo range_m and SNR
                            if rhs_signal.observable == pr_observable {
                                rtk_observations.push(Observation::pseudo_range(
                                    rtk_carrier,
                                    rhs_signal.value,
                                    rhs_signal.snr.map(|snr| snr.into()),
                                ));
                            }

                            // try to attach doppler if we can
                            if rhs_signal.observable.is_doppler_observable() {
                                rtk_observations[0].with_doppler(rhs_signal.value);
                            }
                        } else {
                            // attach subsidary pseudo range (if we have to)
                            if matches!(solver.cfg.method, Method::CPP | Method::PPP) {
                                if rhs_signal.observable.is_pseudo_range_observable() {}
                            }

                            // attach phase range (if we have to)
                            if solver.cfg.method == Method::PPP {
                                if rhs_signal.observable.is_phase_range_observable() {}
                            }
                        }
                    }
                }
            }

            // form candidate to propose
            let mut candidate = Candidate::new(sv, t, rtk_observations);

            // on board clock correction
            match clock.next_clock_at(t, sv) {
                Some(corr) => {
                    candidate.set_clock_correction(corr);
                },
                None => {
                    error!("{} ({}) - no clock correction available", t, sv);
                },
            }

            // on board group delay
            if let Some((_, _, eph)) = eph.borrow_mut().select(t, sv) {
                if let Some(tgd) = eph.tgd() {
                    debug!("{} ({}) - tgd: {}", t, sv, tgd);
                    candidate.set_group_delay(tgd);
                }
            }

            // Tropo
            candidate.set_tropo_components(tropo);

            // Iono
            let iono = if let Some(model) = kb_model(nav_data, t) {
                IonoComponents::KbModel(model)
            } else if let Some(model) = bd_model(nav_data, t) {
                IonoComponents::BdModel(model)
            } else if let Some(model) = ng_model(nav_data, t) {
                IonoComponents::NgModel(model)
            } else {
                // TODO STEC/IONEX
                IonoComponents::Unknown
            };

            // Iono
            candidate.set_iono_components(iono);

            // TODO: RTK
            //candidate.set_remote_observations(remote);

            // resolution attempt
            match solver.resolve(t, &vec![candidate]) {
                Ok((t, pvt_solution)) => {
                    let pvt_data = pvt_solution.sv.get(&sv).unwrap(); // infaillible

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
                        t, sv, pr_observable, elevation, azimuth, refsv, refsys
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

                    let target = &(sv, pr_observable.clone());

                    let tracker = match trackers.get_mut(target) {
                        None => {
                            // initialize new tracker
                            trackers.insert((sv, pr_observable.clone()), SVTracker::default());
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
                                                sv,
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
                                                &pr_observable.to_string(),
                                            )
                                        },
                                        _ => {
                                            Track::new(
                                                sv,
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
                                                &pr_observable.to_string(),
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
        } // unique sv

        next_release = Some(sched.next_track_start(t));
        trk_midpoint = Some(next_release.unwrap() - trk_duration / 2);
        info!("{:?} - {} until next track", t, next_release.unwrap() - t);
    } //.observations(temporal)

    tracks.sort_by(|a, b| a.epoch.cmp(&b.epoch));

    Ok(tracks)
}
