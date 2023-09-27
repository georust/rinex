use nyx_space::cosmic::eclipse::{eclipse_state, EclipseState};
use nyx_space::cosmic::{Orbit, SPEED_OF_LIGHT};
use nyx_space::md::prelude::{Bodies, LightTimeCalc};
use rinex::prelude::{Duration, Epoch, Sv};
use rinex_qc::QcContext;
use std::collections::HashMap;

use hifitime::Unit;

extern crate nyx_space as nyx;

use nalgebra::base::{DVector, Matrix4xX, Vector1, Vector3, Vector4};
use nyx::md::prelude::{Arc, Cosm};

mod estimate;
mod model;
mod opts;

pub mod prelude {
    pub use crate::estimate::SolverEstimate;
    pub use crate::model::Modeling;
    pub use crate::opts::PositioningMode;
    pub use crate::opts::SolverOpts;
    pub use crate::Solver;
    pub use crate::SolverError;
    pub use crate::SolverType;
}

use estimate::SolverEstimate;
use model::Modeling;
use opts::SolverOpts;

use log::{debug, trace, warn};

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum SolverError {
    #[error("provided context is either unsufficient or invalid for any position solving")]
    Unfeasible,
    #[error("apriori position is not defined - initialization it not complete!")]
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
    /// Cosmic model
    cosmic: Arc<Cosm>,
    /// Solver parametrization
    pub opts: SolverOpts,
    /// Whether this solver is initiated (ready to iterate) or not
    initiated: bool,
    /// Type of solver implemented
    pub solver: SolverType,
    /// Current epoch
    nth_epoch: usize,
}

impl Solver {
    pub fn from(context: &QcContext) -> Result<Self, SolverError> {
        let solver = SolverType::from(context)?;
        Ok(Self {
            cosmic: Cosm::de438(),
            solver,
            initiated: false,
            opts: SolverOpts::default(solver),
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

        // 2: interpolate: if need be
        //if !ctx.interpolated {
        //    trace!("orbit interpolation..");
        //    let order = self.opts.interp_order;
        //    ctx.orbit_interpolation(order, None);
        //    //TODO could be nice to have some kind of timing/perf evaluation here
        //    //     and also total number of required interpolations
        //}

        //initialization
        self.nth_epoch = 0;
        /*
         * Solving needs a ref. position
         */
        if self.opts.rcvr_position.is_none() {
            // defined in context ?
            let position = ctx
                .ground_position()
                .ok_or(SolverError::InitializationError(String::from(
                    "Missing ref. position",
                )))?;
            self.opts.rcvr_position = Some(position);
        }

        self.initiated = true;
        Ok(())
    }
    pub fn run(&mut self, ctx: &mut QcContext) -> Result<(Epoch, SolverEstimate), SolverError> {
        if !self.initiated {
            return Err(SolverError::NotInitialized);
        }

        let pos0 = self
            .opts
            .rcvr_position
            .ok_or(SolverError::UndefinedAprioriPosition)?;

        let (x0, y0, z0): (f64, f64, f64) = pos0.into();

        let modeling = self.opts.modeling;

        // grab work instant
        let t = ctx.primary_data().epoch().nth(self.nth_epoch);

        if t.is_none() {
            self.nth_epoch += 1;
            return Err(SolverError::EpochDetermination(self.nth_epoch));
        }

        let t = t.unwrap();
        let interp_order = self.opts.interp_order;

        /* selection */
        let elected_sv = Self::sv_election(ctx, t, &self.opts);
        if elected_sv.is_none() {
            warn!("no vehicles elected @ {}", t);
            self.nth_epoch += 1;
            return Err(SolverError::NoSv(t));
        }

        let mut elected_sv = elected_sv.unwrap();
        debug!("elected sv : {:?}", elected_sv);

        if elected_sv.len() < 4 {
            warn!("not enough vehicles elected");
            self.nth_epoch += 1;
            return Err(SolverError::LessThan4Sv(t));
        }

        /* SV positions */
        /* TODO: SP3 APC corrections: Self::eval_sun_vector3d */
        let mut sv_pos: HashMap<Sv, (f64, f64, f64)> = HashMap::new();
        for sv in &elected_sv {
            if let Some(sp3) = ctx.sp3_data() {
                if let Some((x_km, y_km, z_km)) = sp3.sv_position_interpolate(*sv, t, interp_order)
                {
                    sv_pos.insert(*sv, (x_km, y_km, z_km));
                } else if let Some(nav) = ctx.navigation_data() {
                    if let Some((x_km, y_km, z_km)) =
                        nav.sv_position_interpolate(*sv, t, interp_order)
                    {
                        sv_pos.insert(*sv, (x_km, y_km, z_km));
                    }
                }
            } else {
                if let Some(nav) = ctx.navigation_data() {
                    if let Some((x_km, y_km, z_km)) =
                        nav.sv_position_interpolate(*sv, t, interp_order)
                    {
                        sv_pos.insert(*sv, (x_km, y_km, z_km));
                    }
                }
            }
        }
        /* remove sv in eclipse */
        if self.solver == SolverType::PPP {
            if let Some(min_rate) = self.opts.min_sv_sunlight_rate {
                sv_pos.retain(|sv, (x_km, y_km, z_km)| {
                    let state = self.eclipse_state(*x_km, *y_km, *z_km, t);
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
                    }
                    !eclipsed
                });
            }
        }
        // 3: t_tx
        let mut t_tx: HashMap<Sv, Epoch> = HashMap::new();
        for sv in &elected_sv {
            if let Some(sv_t_tx) = Self::sv_transmission_time(ctx, *sv, t, modeling) {
                t_tx.insert(*sv, sv_t_tx);
            }
        }
        // 4: retrieve rt pseudorange
        let pr: Vec<_> = ctx
            .primary_data()
            .pseudo_range()
            .filter_map(|((epoch, _), sv, _, pr)| {
                if epoch == t && elected_sv.contains(&sv) {
                    Some((sv, pr))
                } else {
                    None
                }
            })
            .collect();

        // 5: position @ tx
        let mut sv_pos: HashMap<Sv, Vector3<f64>> = HashMap::new();
        for t_tx in t_tx {
            if modeling.relativistic_clock_corr {
                // Needs sv_spoeed()
                // -2 * r.dot(v) / c / c
            }
            if modeling.earth_rotation {
                // dt = || rsat - rcvr0 || /c
                // rsat = R3 * we * dt * rsat
                // we = 7.2921151467 E-5
            }
        }

        // 6: form matrix
        let mut y = DVector::<f64>::zeros(elected_sv.len());
        let mut g = Matrix4xX::<f64>::zeros(elected_sv.len());

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
        let estimate = SolverEstimate::new(g, y);
        if estimate.is_none() {
            self.nth_epoch += 1;
            return Err(SolverError::SolvingError(t));
        } else {
            Ok((t, estimate.unwrap()))
        }
    }
    /*
     * Evalutes T_tx transmission time, for given Sv at desired 't'
     */
    fn sv_transmission_time(ctx: &QcContext, sv: Sv, t: Epoch, m: Modeling) -> Option<Epoch> {
        let nav = ctx.navigation_data()?;
        // need one pseudo range observation for this SV @ 't'
        let mut pr = ctx
            .primary_data()
            .pseudo_range()
            .filter_map(|((e, flag), svnn, _, p)| {
                if e == t && flag.is_ok() && svnn == sv {
                    Some(p)
                } else {
                    None
                }
            })
            .take(1);
        if let Some(pr) = pr.next() {
            let t_tx = Duration::from_seconds(t.to_duration().to_seconds() - pr / SPEED_OF_LIGHT);
            let mut e_tx = Epoch::from_duration(t_tx, sv.constellation.timescale()?);

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
        } else {
            debug!("missing PR measurement");
            None
        }
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
     * Elects sv for this epoch
     */
    fn sv_election(ctx: &QcContext, t: Epoch, opts: &SolverOpts) -> Option<Vec<Sv>> {
        const max_sv: usize = 5;
        //TODO: make sure pseudo range exists
        //TODO: make sure context is consistent with solving strategy : SPP / PPP
        ctx.primary_data()
            .sv_epoch()
            .filter_map(|(epoch, svs)| {
                if epoch == t {
                    //if !opts.gnss.is_empty() {
                    //    svs.iter_mut()
                    //        .filter(|sv| opts.gnss.contains(&sv.constellation))
                    //        .count()
                    //} else {
                    // no gnss filter / criteria
                    Some(svs.into_iter().take(max_sv).collect())
                    //}
                } else {
                    None
                }
            })
            .reduce(|svs, _| svs)
    }
}
