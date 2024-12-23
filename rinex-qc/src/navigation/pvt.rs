use log::{error, info};

use std::{cell::RefCell, collections::HashMap};

use crate::{
    context::{meta::MetaData, QcContext},
    navigation::{clock::ClockContext, orbit::OrbitalContext, signal::SignalSource},
    QcError,
};

use itertools::Itertools;

use gnss_rtk::prelude::{
    Candidate, Carrier as RTKCarrier, ClockCorrection, Config as RTKConfig, Epoch,
    Error as RTKError, Frame, Observation, Orbit, PVTSolution, Solver, SV,
};

use rinex::prelude::{obs::SignalObservation, Carrier, Observable};

use super::eph::EphemerisContext;

/// Converts [Carrier] to [RTKCarrier]
fn carrier_to_rtk(carrier: &Carrier) -> Option<RTKCarrier> {
    match carrier {
        Carrier::L1 => Some(RTKCarrier::L1),
        Carrier::L2 => Some(RTKCarrier::L2),
        Carrier::L5 => Some(RTKCarrier::L5),
        Carrier::L6 => Some(RTKCarrier::L6),
        Carrier::E1 => Some(RTKCarrier::E1),
        Carrier::E5 => Some(RTKCarrier::E5),
        Carrier::E6 => Some(RTKCarrier::E6),
        Carrier::E5a => Some(RTKCarrier::E5A),
        Carrier::E5b => Some(RTKCarrier::E5B),
        Carrier::B2 => Some(RTKCarrier::B2),
        Carrier::B2A => Some(RTKCarrier::B2A),
        Carrier::B2B | Carrier::B2I => Some(RTKCarrier::B2iB2b),
        Carrier::B1I => Some(RTKCarrier::B1I),
        Carrier::B1A | Carrier::B1C => Some(RTKCarrier::B1aB1c),
        Carrier::B3 | Carrier::B3A => Some(RTKCarrier::B3),
        Carrier::G1(_) | Carrier::G1a | Carrier::G2(_) | Carrier::G2a | Carrier::G3 => None,
        Carrier::S => None,
        Carrier::S1 | Carrier::U2 => None,
    }
}

fn carrier_from_rtk(carrier: &RTKCarrier) -> Carrier {
    match carrier {
        RTKCarrier::B1I => Carrier::B1I,
        RTKCarrier::B1aB1c => Carrier::B1A,
        RTKCarrier::B2 => Carrier::B2,
        RTKCarrier::B2A => Carrier::B2A,
        RTKCarrier::B2iB2b => Carrier::B2I,
        RTKCarrier::B3 => Carrier::B3,
        RTKCarrier::E1 => Carrier::E1,
        RTKCarrier::E5 => Carrier::E5,
        RTKCarrier::E5A => Carrier::E5a,
        RTKCarrier::E5B => Carrier::E5b,
        RTKCarrier::E6 => Carrier::E6,
        RTKCarrier::L1 => Carrier::L1,
        RTKCarrier::L2 => Carrier::L2,
        RTKCarrier::L5 => Carrier::L5,
        RTKCarrier::L6 => Carrier::L6,
    }
}

/// [NavPvtSolver] is an efficient structure to consume [QcContext]
/// and resolve all possible [PVTSolution]s from it.
pub struct NavPvtSolver<'a> {
    pool: Vec<Candidate>,
    signal: SignalSource<'a>,
    solver: Solver,
    eph_ctx: RefCell<EphemerisContext<'a>>,
    observations: HashMap<RTKCarrier, Observation>,
}

impl<'a> NavPvtSolver<'a> {}

impl<'a> Iterator for NavPvtSolver<'a> {
    type Item = Result<PVTSolution, RTKError>;

    fn next(&mut self) -> Option<Self::Item> {
        let collected = self.signal.collect_epoch();

        if collected.is_none() {
            info!("consumed all signals");
            return None;
        }

        // orbital snapshot
        let orbit = OrbitalContext::new(&self.eph_ctx);

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

            // create pool
            let mut observations = Vec::new();
            for (_, observation) in self.observations.iter() {
                observations.push(observation.clone());
            }

            self.pool.push(Candidate::new(*sv, t, observations));
        }

        // candidate(s) customization(s)
        for cd in self.pool.iter_mut() {
            if let Some((toc, _, eph)) = self.eph_ctx.borrow_mut().select(t, cd.sv) {
                if let Some(dt) = eph.clock_correction(toc, t, cd.sv, 5) {
                    let correction = ClockCorrection::without_relativistic_correction(dt);
                    cd.set_clock_correction(correction);
                } else {
                    error!("{}({}): clock correction", cd.t, cd.sv);
                }
            } else {
                error!("{}({}): ephemeris selection", cd.t, cd.sv);
            }
        }

        panic!(
            "pool: {:?}",
            self.pool.iter().map(|cd| cd.sv).collect::<Vec<_>>()
        );

        // attempt resolution
        match self.solver.resolve(t, &self.pool, orbit) {
            Ok((_, pvt)) => {
                // clear for next time
                self.pool.clear();
                self.observations.clear();

                Some(Ok(pvt))
            },
            Err(e) => {
                error!("rtk error: {}", e);

                // clear for next time
                self.pool.clear();
                self.observations.clear();
                Some(Err(e))
            },
        }
    }
}

impl QcContext {
    /// Create a new [NavPvtSolver] ready to iterate this [QcContext]
    /// and resolve all possible navigation solutions for specifically selected rover.
    pub fn nav_pvt_solver<'a>(
        &'a self,
        cfg: RTKConfig,
        meta: &MetaData,
        initial: Option<Orbit>,
    ) -> Result<NavPvtSolver, QcError> {
        // Obtain ephemeris context
        let eph_ctx = self.ephemeris_context().ok_or(QcError::EphemerisSource)?;

        // Obtain signal source
        let signal = self.signal_source(&meta).ok_or(QcError::SignalSource)?;

        // Deploy solver: share almanac & reference frame model
        let solver = Solver::new_almanac_frame(&cfg, initial, self.almanac.clone(), self.earth_cef);

        Ok(NavPvtSolver {
            solver,
            signal,
            pool: Vec::with_capacity(8),
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

    use gnss_rtk::prelude::Config as RTKConfig;

    #[test]
    pub fn pvt_solver() {
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

        let _ = ctx.nav_pvt_solver(rtk_cfg, &meta, None).unwrap();
    }
}
