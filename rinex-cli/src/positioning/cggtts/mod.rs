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

    let dominant_sampling_period = obs_data
        .dominant_sample_rate()
        .expect("RNX2CGGTTS requires steady GNSS observations");


            // resolution attempt
            match solver.resolve(t, &vec![candidate]) {
                Ok((t, pvt_solution)) => {

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
    } //.observations(temporal)

    tracks.sort_by(|a, b| a.epoch.cmp(&b.epoch));

    Ok(tracks)
}
