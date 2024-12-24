use log::{error, info};

use std::{cell::RefCell, collections::HashMap};

use crate::{
    context::{meta::MetaData, QcContext},
    navigation::{
        carrier_to_rtk, clock::ClockContext, orbit::OrbitalContext, signal::SignalSource,
    },
    QcError, QcRtkCggttsError,
};

use hifitime::Unit;

use itertools::Itertools;

use gnss_rtk::prelude::{
    Candidate, Carrier as RTKCarrier, ClockCorrection, Config as RTKConfig, Duration, Epoch,
    Error as RTKError, Frame, IonosphereBias as RTKIonosphereBias, Observation, Orbit, PVTSolution,
    Solver, SPEED_OF_LIGHT_M_S, SV,
};

use rinex::prelude::{obs::SignalObservation, Carrier, Observable};

use super::eph::EphemerisContext;

use cggtts::{prelude::Track as CggttsTrack, track::Scheduler as CggttsScheduler};

use anise::math::Vector6;

/// [NavCggttsSolver] is an efficient structure to consume [QcContext]
/// and resolve all possible CGGTTS [Track]s from it.
pub struct NavCggttsSolver<'a> {
    signal: SignalSource<'a>,
    solver: Solver,
    eph_ctx: RefCell<EphemerisContext<'a>>,
    observations: HashMap<RTKCarrier, Observation>,
    /// Track scheduling table
    scheduler: CggttsScheduler,
    /// Epoch of next publication
    next_release: Epoch,
    /// Next track midpoint
    track_midpoint: Epoch,
}

impl<'a> Iterator for NavCggttsSolver<'a> {
    type Item = Result<CggttsTrack, QcRtkCggttsError>;

    fn next(&mut self) -> Option<Self::Item> {
        let collected = self.signal.collect_epoch();

        if collected.is_none() {
            info!("consumed all signals");
            return None;
        }

        // clock snapshot
        let clock = ClockContext::new(&self.eph_ctx);

        // gather candidates
        let (t, signals) = collected.unwrap();

        let sv_list = signals
            .iter()
            .map(|sig| sig.sv)
            .unique()
            .collect::<Vec<_>>();

        // per unique SV
        for sv in sv_list.iter() {
            self.observations.clear();

            // per unique carrier
            for carrier in signals
                .iter()
                .filter_map(|sig| {
                    if sig.sv == *sv {
                        if let Ok(carrier) = sig.observable.carrier(sig.sv.constellation) {
                            carrier_to_rtk(&carrier)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unique()
            {
                // gather observations
                for signal in signals.iter().filter_map(|sig| {
                    if sig.sv == *sv {
                        let mut is_interesting = sig.observable.is_phase_range_observable();
                        is_interesting |= sig.observable.is_pseudo_range_observable();
                        is_interesting |= sig.observable.is_doppler_observable();
                        is_interesting |= sig.observable.is_ssi_observable();

                        if is_interesting {
                            Some(sig)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }) {
                    match signal.observable {
                        Observable::PhaseRange(_) => {
                            if let Some(observation) = self.observations.get_mut(&carrier) {
                                observation.phase = Some(signal.value);
                            } else {
                                self.observations.insert(
                                    carrier,
                                    Observation::ambiguous_phase_range(carrier, signal.value, None),
                                );
                            }
                        },
                        Observable::Doppler(_) => {
                            if let Some(observation) = self.observations.get_mut(&carrier) {
                                observation.doppler = Some(signal.value);
                            } else {
                                self.observations.insert(
                                    carrier,
                                    Observation::doppler(carrier, signal.value, None),
                                );
                            }
                        },
                        Observable::PseudoRange(_) => {
                            if let Some(observation) = self.observations.get_mut(&carrier) {
                                observation.pseudo = Some(signal.value);
                            } else {
                                self.observations.insert(
                                    carrier,
                                    Observation::pseudo_range(carrier, signal.value, None),
                                );
                            }
                        },
                        Observable::SSI(_) => {
                            if let Some(observation) = self.observations.get_mut(&carrier) {
                                observation.snr = Some(signal.value);
                            }
                        },
                        _ => unreachable!("filtered out"),
                    }
                }
            }

            // create candidate
            let mut observations = Vec::new();
            for (_, observation) in self.observations.iter() {
                observations.push(observation.clone());
            }

            let mut candidate = Candidate::new(*sv, t, observations);

            if let Some((toc, toe, eph)) = self.eph_ctx.borrow_mut().select(t, candidate.sv) {
                if let Some(tgd) = eph.tgd() {
                    candidate.set_group_delay(tgd);
                }

                if let Some(dt) = eph.clock_correction(toc, t, *sv, 5) {
                    let correction = ClockCorrection::without_relativistic_correction(dt);
                    candidate.set_clock_correction(correction);
                } else {
                    error!("{}({}): clock correction", t, *sv);
                }
            } else {
                error!("{}({}): ephemeris selection", t, *sv);
            }

            // orbital snapshot
            let orbit = OrbitalContext::new(&self.eph_ctx);

            // attempt resolution
            match self.solver.resolve(t, &[candidate], orbit) {
                Ok((_, pvt_solution)) => {
                    // clear for next time
                    let sv_pvt = pvt_solution.sv.get(&sv).unwrap(); // infaillible @ this point

                    let (azimuth_deg, elevation_deg) = (sv_pvt.azimuth, sv_pvt.elevation);

                    let refsys = pvt_solution.dt.to_seconds();

                    let correction = sv_pvt.clock_correction.unwrap_or_default();

                    let refsv = refsys + correction.to_seconds();

                    // tropod always exists in CGGTTS
                    let mdtr = sv_pvt.tropo_bias.unwrap_or_default() / SPEED_OF_LIGHT_M_S;

                    let mdio = match sv_pvt.iono_bias {
                        Some(RTKIonosphereBias::Modeled(bias)) => Some(bias),
                        _ => None,
                    };

                    let msio = match sv_pvt.iono_bias {
                        Some(RTKIonosphereBias::Measured(bias)) => Some(bias),
                        _ => None,
                    };

                    info!("{:?}({}): new solution: (azim={:.2}°, elev={:.2}°, refsv={:.3E}, refsys={:.3E})", t, sv, azimuth_deg, elevation_deg, refsv, refsys);

                    // // form FitData
                    // let fitdata = FitData {
                    //     refsv,
                    //     refsys,
                    //     mdtr,
                    //     mdio,
                    //     msio,
                    //     azimuth_deg,
                    //     elevation_deg,
                    // };
                },
                Err(e) => {
                    error!("rtk error: {}", e);

                    // clear for next time
                    self.observations.clear();
                },
            }

            self.observations.clear();
        }

        Some(Err(QcRtkCggttsError::Dumy))
    }
}

impl QcContext {
    /// Create a new [NavCggttsSolver] ready to iterate this [QcContext]
    /// and resolve all possible CGGTTS solutions for specifically selected rover.
    /// ## Inputs
    /// - cfg: [RTKConfig] setup
    /// - meta: [MetaData] rover selector
    /// - rx_position_ecef_m: mandatory ground position expressed in ECEF (km)
    /// - tracking: [Duration] to be used by track scheduler
    pub fn nav_cggtts_solver<'a>(
        &'a self,
        cfg: RTKConfig,
        meta: &MetaData,
        rx_position_ecef_km: Option<(f64, f64, f64)>,
        tracking_duration: Duration,
    ) -> Result<NavCggttsSolver, QcError> {
        // Obtain ephemeris context
        let eph_ctx = self.ephemeris_context().ok_or(QcError::EphemerisSource)?;

        // Obtain signal source
        let signal = self.signal_source(&meta).ok_or(QcError::SignalSource)?;
        let rinex = self.obs_dataset.get(&meta).ok_or(QcError::RxPosition)?;

        // Reference position: prefer user settings over RINex position
        let pos_vel = if let Some((x_ecef_km, y_ecef_km, z_ecef_km)) = rx_position_ecef_km {
            Vector6::new(x_ecef_km, y_ecef_km, z_ecef_km, 0.0, 0.0, 0.0)
        } else {
            // Using internal position (which then, needs to be defined)
            let (x_ecef_m, y_ecef_m, z_ecef_m) =
                rinex.header.rx_position.ok_or(QcError::RxPosition)?;

            info!(
                "using RINex ({}) reference position: {:?}",
                meta,
                (x_ecef_m, y_ecef_m, z_ecef_m)
            );

            Vector6::new(
                x_ecef_m / 1000.0,
                y_ecef_m / 1000.0,
                z_ecef_m / 1000.0,
                0.0,
                0.0,
                0.0,
            )
        };

        // Position of the receiver
        let t0 = rinex.first_epoch().ok_or(QcError::RxPosition)?;

        let rx_orbit = Orbit::from_cartesian_pos_vel(pos_vel, t0, self.earth_cef);

        // Deploy solver: share almanac & reference frame model
        let solver =
            Solver::new_almanac_frame(&cfg, Some(rx_orbit), self.almanac.clone(), self.earth_cef);

        // Initialize the track scheduler
        let scheduler = CggttsScheduler::new(tracking_duration);
        let next_release = scheduler.next_track_start(t0);
        let track_midpoint =
            next_release - (3.0 * 60.0) * Unit::Second - (780.0 * Unit::Second) / 2.0;

        info!("{}: {} until next track", t0, next_release - t0);

        Ok(NavCggttsSolver {
            solver,
            signal,
            scheduler,
            next_release,
            track_midpoint,
            eph_ctx: RefCell::new(eph_ctx),
            observations: HashMap::with_capacity(8),
        })
    }
}

#[cfg(test)]
mod test {

    use crate::{
        cfg::QcConfig,
        context::{meta::MetaData, QcContext},
    };

    use gnss_rtk::prelude::{Config as RTKConfig, Duration, Orbit};

    #[test]
    pub fn cggtts_solver() {
        let cfg = QcConfig::default();

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_gzip_file(&format!(
            "{}/../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
            env!("CARGO_MANIFEST_DIR"),
        ))
        .unwrap();

        ctx.load_gzip_file(&format!(
            "{}/../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz",
            env!("CARGO_MANIFEST_DIR"),
        ))
        .unwrap();

        let rtk_cfg = RTKConfig::default();

        let meta = MetaData {
            name: "ESBC00DNK".to_string(),
            extension: "crx.gz".to_string(),
            unique_id: Some("rcvr:SEPT POLARX5".to_string()),
        };

        let _ = ctx
            .nav_cggtts_solver(rtk_cfg, &meta, None, Duration::from_seconds(60.0))
            .unwrap();
    }
}
