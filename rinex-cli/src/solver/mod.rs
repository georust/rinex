use nyx_space::cosmic::eclipse::{eclipse_state, EclipseState};
use nyx_space::cosmic::{Orbit, SPEED_OF_LIGHT};
use nyx_space::md::prelude::{Arc, Bodies, Cosm, LightTimeCalc};
use rinex::prelude::{Duration, Epoch, Sv};
use rinex_qc::QcContext;
use std::collections::HashMap;
use thiserror::Error;

mod opts;
use opts::{PositioningMode, SolverOpts};

mod models;

#[derive(Debug, Clone, Copy, Error)]
pub enum Error {
    #[error("provided context is either unsufficient or invalid for any position solving")]
    Unfeasible,
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
    fn from(ctx: &QcContext) -> Result<Self, Error> {
        if ctx.primary_data().is_observation_rinex() {
            if ctx.has_sp3() {
                Ok(Self::PPP)
            } else {
                if ctx.has_navigation_data() {
                    Ok(Self::SPP)
                } else {
                    Err(Error::Unfeasible)
                }
            }
        } else {
            Err(Error::Unfeasible)
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
    /// current estimate
    pub estimate: Estimate,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Estimate {
    /// Position estimate
    pub pos: (f64, f64, f64),
    /// Time offset estimate
    pub clock_offset: Duration,
}

impl Solver {
    pub fn from(context: &QcContext) -> Result<Self, Error> {
        let solver = SolverType::from(context)?;
        Ok(Self {
            cosmic: Cosm::de438(),
            solver,
            initiated: false,
            opts: SolverOpts::default(solver),
            nth_epoch: 0,
            estimate: Estimate::default(),
        })
    }
    pub fn init(&mut self, ctx: &mut QcContext) {
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
        self.estimate.pos = self.opts.rcvr_position.into();
        self.initiated = true;
    }
    pub fn run(&mut self, ctx: &mut QcContext) -> Option<(Epoch, Estimate)> {
        if !self.initiated {
            self.init(ctx);
            trace!("solver initiated");
        } else {
            // move on to next epoch
            self.nth_epoch += 1;
        }

        // grab work instant
        let t = ctx.primary_data().epoch().nth(self.nth_epoch)?;

        let interp_order = self.opts.interp_order;

        /* elect vehicles */
        let mut elected_sv = Self::sv_election(ctx, t);
        if elected_sv.is_none() {
            warn!("no vehicles elected @ {}", t);
            return Some((t, self.estimate));
        }

        let mut elected_sv = elected_sv.unwrap();
        debug!("elected sv : {:?}", elected_sv);

        /* determine sv positions */
        /* TODO: SP3 APC corrections */
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
            sv_pos.retain(|_, (x_km, y_km, z_km)| !self.eclipsed(*x_km, *y_km, *z_km, t));
        }

        // 3: t_tx
        let mut t_tx: HashMap<Sv, Epoch> = HashMap::new();
        for sv in &elected_sv {
            if let Some(sv_t_tx) = Self::sv_transmission_time(ctx, *sv, t) {
                t_tx.insert(*sv, sv_t_tx);
            }
        }

        //TODO
        // add other models

        // form matrix
        // resolve

        Some((t, self.estimate))
    }
    /*
     * Evalutes T_tx transmission time, for given Sv at desired 't'
     */
    fn sv_transmission_time(ctx: &QcContext, sv: Sv, t: Epoch) -> Option<Epoch> {
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
            debug!("t_tx(pr): {}@{} : {}", sv, t, t_tx);

            let mut e_tx = Epoch::from_duration(t_tx, sv.constellation.timescale()?);
            let dt_sat = nav.sv_clock_bias(sv, e_tx)?;
            debug!("clock bias: {}@{} : {}", sv, t, dt_sat);

            e_tx -= dt_sat;
            debug!("{} : t(obs): {} | t(tx) {}", sv, t, e_tx);
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
    fn eclipsed(&self, x_km: f64, y_km: f64, z_km: f64, epoch: Epoch) -> bool {
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
        eclipse_state(&sv_orbit, sun_frame, earth_frame, &self.cosmic) == EclipseState::Umbra
    }
    /*
     * Elects sv for this epoch
     */
    fn sv_election(ctx: &QcContext, t: Epoch) -> Option<Vec<Sv>> {
        ctx.primary_data()
            .sv_epoch()
            .filter_map(|(epoch, svs)| if epoch == t { Some(svs) } else { None })
            .reduce(|svs, _| svs)
    }
}
