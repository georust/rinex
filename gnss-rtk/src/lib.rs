use nyx_space::cosmic::eclipse::{eclipse_state, EclipseState};
use nyx_space::cosmic::{Orbit, SPEED_OF_LIGHT};
use nyx_space::md::prelude::{Bodies, LightTimeCalc};
use rinex::prelude::{Duration, Epoch, Sv};
use rinex_qc::QcContext;
use std::collections::HashMap;

use hifitime::Unit;

extern crate nyx_space as nyx;

use nalgebra::base::{DVector, MatrixXx4, Vector1, Vector3, Vector4};
use nalgebra::vector;
use nyx::md::prelude::{Arc, Cosm};

mod cfg;
mod estimate;
mod model;

pub mod prelude {
    pub use crate::cfg::RTKConfig;
    pub use crate::cfg::SolverMode;
    pub use crate::estimate::SolverEstimate;
    pub use crate::model::Modeling;
    pub use crate::Solver;
    pub use crate::SolverError;
    pub use crate::SolverType;
}

use cfg::RTKConfig;
use estimate::SolverEstimate;
use model::Modeling;

use log::{debug, trace, warn};

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum SolverError {
    #[error("provided context is either not sufficient or invalid")]
    Unfeasible,
    #[error("apriori position is not defined")]
    UndefinedAprioriPosition,
    #[error("failed to initialize solver - \"{0}\"")]
    InitializationError(String),
    #[error("no vehicles elected @{0}")]
    NoSv(Epoch),
    #[error("not enough vehicles elected @{0}")]
    LessThan4Sv(Epoch),
    #[error("failed to retrieve work epoch (index: {0})")]
    EpochDetermination(usize),
    #[error("badop: solver not initialized")]
    NotInitialized,
    #[error("failed to invert navigation matrix @{0}")]
    SolvingError(Epoch),
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum SolverType {
    /// SPP : code based and approximated models
    /// aiming a metric resolution.
    #[default]
    SPP,
    /// PPP : phase + code based, the ultimate solver
    /// aiming a millimetric resolution.
    PPP,
}

impl std::fmt::Display for SolverType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SPP => write!(fmt, "SPP"),
            Self::PPP => write!(fmt, "PPP"),
        }
    }
}

impl SolverType {
    fn from(ctx: &QcContext) -> Result<Self, SolverError> {
        if ctx.primary_data().is_observation_rinex() {
            //TODO : multi carrier for selected constellations
            Ok(Self::SPP)
        } else {
            Err(SolverError::Unfeasible)
        }
    }
}

#[derive(Debug)]
pub struct Solver {
    /// Solver parametrization
    pub cfg: RTKConfig,
    /// Type of solver implemented
    pub solver: SolverType,
    /// cosmic model
    cosmic: Arc<Cosm>,
    /// true if self has been initiated and is ready to compute
    initiated: bool,
    /// current epoch
    nth_epoch: usize,
}

impl Solver {
    pub fn from(context: &QcContext) -> Result<Self, SolverError> {
        let solver = SolverType::from(context)?;
        Ok(Self {
            cosmic: Cosm::de438(),
            solver,
            initiated: false,
            cfg: RTKConfig::default(solver),
            nth_epoch: 0,
        })
    }
    pub fn init(&mut self, ctx: &mut QcContext) -> Result<(), SolverError> {
        trace!("{} solver initialization..", self.solver);
        //TODO: Preprocessing:
        //      only for ppp solver
        //      preseve "complete" epochs only
        //        incomplete epochs will not be considered by PPP solver
        //        this reduces nb of Epochs to interpolate
        // trace!("\"complete\" epoch filter..");

        // let total = ctx.primary_data().epoch().count();
        // ctx.complete_epoch_filter_mut(None);
        // let total_dropped = total - ctx.primary_data().epoch().count();
        // trace!(
        //     "dropped a total of {}/{} \"incomplete\" epochs",
        //     total_dropped,
        //     total
        // );
        /*
         * Solving needs a ref. position
         */
        if self.cfg.rcvr_position.is_none() {
            // defined in context ?
            let position = ctx.ground_position();
            if let Some(position) = position {
                self.cfg.rcvr_position = Some(position);
            } else {
                return Err(SolverError::UndefinedAprioriPosition);
            }
        }

        self.nth_epoch = 0;
        self.initiated = true;
        Ok(())
    }
    pub fn run(&mut self, ctx: &mut QcContext) -> Result<(Epoch, SolverEstimate), SolverError> {
        if !self.initiated {
            return Err(SolverError::NotInitialized);
        }

        let pos0 = self
            .cfg
            .rcvr_position
            .ok_or(SolverError::UndefinedAprioriPosition)?;

        let (x0, y0, z0): (f64, f64, f64) = pos0.into();

        let modeling = self.cfg.modeling;
        let interp_order = self.cfg.interp_order;

        // 0: grab work instant
        let t = ctx.primary_data().epoch().nth(self.nth_epoch);

        if t.is_none() {
            self.nth_epoch += 1;
            return Err(SolverError::EpochDetermination(self.nth_epoch));
        }
        let t = t.unwrap();

        // 1: elect sv
        let sv = Self::sv_at_epoch(ctx, t);
        if sv.is_none() {
            warn!("no vehicles found @ {}", t);
            self.nth_epoch += 1;
            return Err(SolverError::NoSv(t));
        }

        let mut elected_sv: Vec<Sv> = sv.unwrap().into_iter().take(self.cfg.max_sv).collect();

        trace!("{}: {} candidates", t, elected_sv.len());

        // retrieve associated PR
        let pr: Vec<_> = ctx
            .primary_data()
            .pseudo_range_ok()
            .filter_map(|(epoch, svnn, _, pr)| {
                if epoch == t && elected_sv.contains(&svnn) {
                    Some((svnn, pr))
                } else {
                    None
                }
            })
            .collect();

        // apply mask filters
        elected_sv.retain(|sv| {
            let has_pr = pr
                .iter()
                .filter_map(|(svnn, pr)| if svnn == sv { Some(pr) } else { None })
                .reduce(|pr, _| pr)
                .is_some();

            if !has_pr {
                trace!("{}@{} : no pseudo range", sv, t);
            }

            let mut ppp_ok = !(self.solver == SolverType::PPP);
            if self.solver == SolverType::PPP {
                //TODO: verify PPP compliancy
            }
            if !ppp_ok {
                trace!("{}@{} : not ppp compliant", sv, t);
            }

            let mut elev_ok = self.cfg.min_sv_elev.is_none();
            if let Some(min_elev) = self.cfg.min_sv_elev {
                if let Some(sp3) = ctx.sp3_data() {
                    //TODO/ elev from SP3 ?
                } else if let Some(nav) = ctx.navigation_data() {
                    //TODO TODO TODO
                    // this will not work as context may not be alined in time
                    // we should already have interpolated at this point
                    //TODO TODO TODO
                    let e = nav
                        .sv_elevation_azimuth(Some(pos0))
                        .filter_map(|(epoch, (svnn, (e, _)))| {
                            if epoch == t && svnn == *sv {
                                Some(e)
                            } else {
                                None
                            }
                        })
                        .reduce(|e, _| e);
                    if let Some(e) = e {
                        elev_ok = e >= min_elev;
                    }
                }
            }
            let mut snr_ok = self.cfg.min_sv_snr.is_none();
            if let Some(min_snr) = self.cfg.min_sv_snr {
                let snr = ctx
                    .primary_data()
                    .snr()
                    .filter_map(|((epoch, _), svnn, _, snr)| {
                        if epoch == t && svnn == *sv {
                            Some(snr)
                        } else {
                            None
                        }
                    })
                    .reduce(|snr, _| snr);
                if let Some(snr) = snr {
                    snr_ok = snr >= min_snr;
                }
            }
            has_pr && elev_ok && snr_ok & ppp_ok
        });

        // make sure we still have enough SV
        if elected_sv.len() < 4 {
            warn!("not enough vehicles elected");
            self.nth_epoch += 1;
            return Err(SolverError::LessThan4Sv(t));
        }

        debug!("elected sv : {:?}", elected_sv);

        // 3: t_tx
        let mut t_tx: HashMap<Sv, Epoch> = HashMap::new();
        for sv in &elected_sv {
            // pr for this SV @ t
            let pr = pr
                .iter()
                .filter_map(|(svnn, pr)| if svnn == sv { Some(*pr) } else { None })
                .reduce(|pr, _| pr);
            // eval
            if let Some(pr) = pr {
                if let Some(sv_t_tx) = Self::sv_transmission_time(ctx, *sv, t, pr, modeling) {
                    t_tx.insert(*sv, sv_t_tx);
                }
            }
        }

        // 4: sv position @ tx
        let mut sv_pos: HashMap<Sv, Vector3<f64>> = HashMap::new();
        for (sv, t_tx) in &t_tx {
            // relativistic clock offset correction
            let t_tx = match modeling.relativistic_clock_corr {
                true => {
                    //TODO
                    let e = 1.204112719279E-2;
                    let sqrt_a = 5.153704689026E3;
                    let sqrt_mu = (3986004.418E8_f64).sqrt();
                    //let dt = -2.0_f64 * sqrt_a * sqrt_mu / SPEED_OF_LIGHT / SPEED_OF_LIGHT * e * elev.sin();
                    t_tx //TODO
                },
                false => t_tx,
            };
            if let Some(sp3) = ctx.sp3_data() {
                //TODO : SP3 needs APC correction
                //    which needs Self::eval_sun_vector3D
                if let Some((x_km, y_km, z_km)) =
                    sp3.sv_position_interpolate(*sv, *t_tx, interp_order)
                {
                    let mut pos = vector![x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3];
                    if modeling.earth_rotation {
                        //TODO
                        // dt = || rsat - rcvr0 || /c
                        // rsat = R3 * we * dt * rsat
                        // we = 7.2921151467 E-5
                    }
                    // Eclipse filter
                    if self.solver == SolverType::PPP {
                        if let Some(min_rate) = self.cfg.min_sv_sunlight_rate {
                            let state = self.eclipse_state(x_km, y_km, z_km, *t_tx);
                            let eclipsed = match state {
                                EclipseState::Umbra => true,
                                EclipseState::Visibilis => false,
                                EclipseState::Penumbra(r) => {
                                    debug!("{} state: {}", sv, state);
                                    r < min_rate
                                },
                            };
                            if eclipsed {
                                debug!("dropping eclipsed {}", sv);
                            } else {
                                sv_pos.insert(*sv, pos);
                            }
                        } else {
                            sv_pos.insert(*sv, pos);
                        }
                    } else {
                        sv_pos.insert(*sv, pos);
                    }
                } else {
                    debug!("{}@{}: sp3 interpolation failed", t, sv);
                }
            } else if let Some(nav) = ctx.navigation_data() {
                if let Some((x_km, y_km, z_km)) =
                    nav.sv_position_interpolate(*sv, *t_tx, interp_order)
                {
                    let mut pos = vector![x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3];
                    if modeling.earth_rotation {
                        //TODO
                        // dt = || rsat - rcvr0 || /c
                        // rsat = R3 * we * dt * rsat
                        // we = 7.2921151467 E-5
                    }
                    // Eclipse filter
                    if self.solver == SolverType::PPP {
                        if let Some(min_rate) = self.cfg.min_sv_sunlight_rate {
                            let state = self.eclipse_state(x_km, y_km, z_km, *t_tx);
                            let eclipsed = match state {
                                EclipseState::Umbra => true,
                                EclipseState::Visibilis => false,
                                EclipseState::Penumbra(r) => {
                                    debug!("{} state: {}", sv, state);
                                    r < min_rate
                                },
                            };
                            if eclipsed {
                                debug!("dropping eclipsed {}", sv);
                            } else {
                                sv_pos.insert(*sv, pos);
                            }
                        } else {
                            sv_pos.insert(*sv, pos);
                        }
                    } else {
                        sv_pos.insert(*sv, pos);
                    }
                } else {
                    debug!("{}@{}: kepler interpolation failed", t, sv);
                }
            }
        }

        // 6: form matrix
        let mut y = DVector::<f64>::zeros(elected_sv.len());
        let mut g = MatrixXx4::<f64>::zeros(elected_sv.len());

        for (index, (sv, pos)) in sv_pos.iter().enumerate() {
            let rho_0 =
                ((pos[0] - x0).powi(2) + (pos[1] - y0).powi(2) + (pos[2] - z0).powi(2)).sqrt();
            //TODO
            //let models = models
            //    .iter()
            //    .filter_map(|sv, model| {
            //        if sv == svnn {
            //            Some(model)
            //        } else {

            //        }
            //    })
            //    .reduce(|m, _| m)
            //    .unwrap();
            let models = 0.0_f64;

            let pr = pr
                .iter()
                .filter_map(|(svnn, pr)| if sv == svnn { Some(pr) } else { None })
                .reduce(|pr, _| pr)
                .unwrap();

            y[index] = pr - rho_0 - models;

            let (x, y, z) = (pos[0], pos[1], pos[2]);

            g[(index, 0)] = (x0 - x) / rho_0;
            g[(index, 1)] = (y0 - y) / rho_0;
            g[(index, 2)] = (z0 - z) / rho_0;
            g[(index, 3)] = 1.0_f64;
        }

        // 7: resolve
        trace!("y: {} | g: {}", y, g);
        let estimate = SolverEstimate::new(g, y);
        self.nth_epoch += 1;

        if estimate.is_none() {
            return Err(SolverError::SolvingError(t));
        } else {
            Ok((t, estimate.unwrap()))
        }
    }
    /*
     * Evalutes T_tx transmission time, for given Sv at desired 't'
     */
    fn sv_transmission_time(
        ctx: &QcContext,
        sv: Sv,
        t: Epoch,
        pr: f64,
        m: Modeling,
    ) -> Option<Epoch> {
        let nav = ctx.navigation_data()?;
        let ts = sv.constellation.timescale()?;
        let seconds_ts = t.to_duration_in_time_scale(ts).to_seconds();
        let mut e_tx = Epoch::from_duration((seconds_ts - pr / SPEED_OF_LIGHT) * Unit::Second, ts);

        if m.sv_clock_bias {
            let dt_sat = nav.sv_clock_bias(sv, e_tx)?;
            debug!("{}@{} | dt_sat {}", sv, t, dt_sat);
            e_tx -= dt_sat;
        }

        if m.sv_total_group_delay {
            if let Some(nav) = ctx.navigation_data() {
                if let Some(tgd) = nav.sv_tgd(sv, t) {
                    let tgd = tgd * Unit::Second;
                    debug!("{}@{} | tgd    {}", sv, t, tgd);
                    e_tx = e_tx - tgd;
                }
            }
        }

        debug!("{}@{} | t_tx    {}", sv, t, e_tx);
        Some(e_tx)
    }
    /*
     * Evaluates Sun/Earth vector, <!> expressed in Km <!>
     * for all SV NAV Epochs in provided context
     */
    #[allow(dead_code)]
    fn eval_sun_vector3d(&mut self, ctx: &QcContext, t: Epoch) -> (f64, f64, f64) {
        let sun_body = Bodies::Sun;
        let eme_j2000 = self.cosmic.frame("EME2000");
        let orbit =
            self.cosmic
                .celestial_state(sun_body.ephem_path(), t, eme_j2000, LightTimeCalc::None);
        (orbit.x_km, orbit.y_km, orbit.z_km)
    }
    /*
     * Computes celestial angle condition
     */
    fn eclipse_state(&self, x_km: f64, y_km: f64, z_km: f64, epoch: Epoch) -> EclipseState {
        let sun_frame = self.cosmic.frame("Sun J2000");
        let earth_frame = self.cosmic.frame("EME2000");
        let sv_orbit = Orbit {
            x_km,
            y_km,
            z_km,
            vx_km_s: 0.0_f64,
            vy_km_s: 0.0_f64,
            vz_km_s: 0.0_f64,
            epoch,
            frame: earth_frame,
            stm: None,
        };
        eclipse_state(&sv_orbit, sun_frame, earth_frame, &self.cosmic)
    }
    /*
     * Returns all Sv at "t"
     */
    fn sv_at_epoch(ctx: &QcContext, t: Epoch) -> Option<Vec<Sv>> {
        ctx.primary_data()
            .sv_epoch()
            .filter_map(|(epoch, svs)| if epoch == t { Some(svs) } else { None })
            .reduce(|svs, _| svs)
    }
}
