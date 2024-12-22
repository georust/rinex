use log::info;
use std::collections::HashMap;

use crate::{
    context::{meta::MetaData, QcContext},
    navigation::eph::EphemerisContext,
    QcError,
};

use itertools::Itertools;

use gnss_rtk::prelude::{
    Candidate, Carrier as RTKCarrier, Config as RTKConfig, Epoch, Frame, Observation, Orbit,
    PVTSolution, Solver, SV,
};

use rinex::prelude::{obs::SignalObservation, Carrier, Observable};

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

pub struct OrbitSource<'a> {
    eph_ctx: EphemerisContext<'a>,
}

impl<'a> gnss_rtk::prelude::OrbitSource for OrbitSource<'a> {
    fn next_at(&mut self, t: Epoch, sv: SV, fr: Frame, order: usize) -> Option<Orbit> {
        let (toc, _, sel_eph) = self.eph_ctx.select(t, sv)?;
        sel_eph.kepler2position(sv, toc, t)
    }
}

/// [NavPvtSolver] is an efficient structure to consume [QcContext]
/// and resolve all possible [PVTSolution]s from it.
pub struct NavPvtSolver<'a> {
    t: Epoch,
    eos: bool,
    pool: Vec<Candidate>,
    rtk_solver: Solver<OrbitSource<'a>>,
    signal_buffer: Vec<SignalObservation>,
    observations: HashMap<RTKCarrier, Observation>,
    signal_iter: Box<dyn Iterator<Item = (Epoch, &'a SignalObservation)> + 'a>,
}

impl<'a> NavPvtSolver<'a> {}

impl<'a> Iterator for NavPvtSolver<'a> {
    type Item = PVTSolution;

    fn next(&mut self) -> Option<Self::Item> {
        // gather signals from ongoing epoch
        let mut t = Option::<Epoch>::None;

        loop {
            if let Some((k, v)) = self.signal_iter.next() {
                if t.is_none() {
                    t = Some(k);
                }

                let t = t.unwrap();
                if k > t {
                    // new epoch: abort
                    // TODO: this simplistic implementation looses one observation
                    // per new epoch
                    break;
                }

                self.signal_buffer.push(v.clone());
            
            } else {
                // EOS
                info!("consumed all signals");
                return None;
            }
        }

        let t = t.unwrap();

        // clear residues from past attempt
        self.pool.clear();
        self.signal_buffer.clear();

        // form all possible candidates
        let sv_list = self
            .signal_buffer
            .iter()
            .map(|sig| sig.sv)
            .unique()
            .collect::<Vec<_>>();

        // per SV
        for sv in sv_list.iter() {

            self.observations.clear();

            // per carrier signals
            for carrier in self
                .signal_buffer
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

                for signal in self.signal_buffer.iter().filter_map(|sig| {
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

                // append to queue
                for (_, observation) in self.observations.iter() {
                    self.candidates.push(Candidate::new(*sv, t, self.observations));
                }

            }
                
        }

        // attempt resolution
        match self.rtk_solver.resolve(t, &self.pool) {
            Ok((_, pvt)) => {
                Some(pvt)
            },
            Err(e) => {
                error!("rtk error: {}", e);
                None
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
        // Obtain orbital source
        let eph_ctx = self.ephemeris_context().ok_or(QcError::NoEphemeris)?;
        let orbit = OrbitSource { eph_ctx };

        // Obtain signal source
        let signal_src = self.obs_dataset.get(&meta).ok_or(QcError::NoSignal)?;
        let t = signal_src.first_epoch().ok_or(QcError::SignalSourceInit)?;
        let signal_iter = signal_src.signal_observations_sampling_ok_iter();

        // Deploy solver: share almanac & reference frame model
        let rtk_solver =
            Solver::new_almanac_frame(&cfg, initial, orbit, self.almanac.clone(), self.earth_cef);

        Ok(NavPvtSolver {
            t,
            eos: false,
            rtk_solver,
            signal_iter,
            pool: Vec::with_capacity(8),
            observations: HashMap::with_capacity(8),
            signal_buffer: Vec::with_capacity(32),
        })
    }
}
