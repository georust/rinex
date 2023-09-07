use rinex::prelude::{Epoch, Sv};
use rinex::preprocessing::{Filter, Preprocessing};
use rinex_qc::QcContext;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub(crate) enum Error {
    #[error("provided context is either unsufficient or invalid for any position solving")]
    Unfeasible,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub(crate) enum SolverType {
    /// SPP : code based
    #[default]
    SPP,
    /// PPP : phase + code based, the ultimate
    PPP,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub(crate) struct SolverOpts {
    /// Criteria (for convergence)
    pub epsilon: f64,
    /// (Position) interpolation filters order
    pub interp_order: usize,
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Solver {
    /// Solver parametrization
    opts: SolverOpts,
    /// Earth/Sun vector, for each NAV Epoch
    sun: HashMap<Epoch, (f64, f64, f64)>,
    /// Type of solver implemented
    pub solver: SolverType,
    /// Current position estimate
    pub estimated_pos: (f64, f64, f64),
    /// Current Epoch estimate
    pub estimated_time: Epoch,
}

use nyx_space::md::prelude::Bodies;
use nyx_space::md::prelude::Cosm;
use nyx_space::md::prelude::LightTimeCalc;

impl Solver {
    pub fn from(context: &QcContext) -> Result<Self, Error> {
        let solver = SolverType::from(context)?;
        Ok(Self {
            opts: SolverOpts::default(),
            solver,
            sun: Self::sun_vector3d(context, solver),
            estimated_pos: (0.0_f64, 0.0_f64, 0.0_f64),
            estimated_time: Epoch::default(),
        })
    }
    pub fn run(&mut self) -> ((f64, f64, f64), Epoch) {
        (self.estimated_pos, self.estimated_time)
    }
    /*
     * Evaluates Sun/Earth vector, <!> expressed in Km <!>
     * for all SV NAV Epochs in provided context
     */
    fn sun_vector3d(ctx: &QcContext, solver: SolverType) -> HashMap<Epoch, (f64, f64, f64)> {
        let mut ret: HashMap<Epoch, (f64, f64, f64)> = HashMap::new();
        let epochs: Vec<Epoch> = match solver {
            SolverType::SPP => ctx.navigation_data().unwrap().epoch().collect(),
            SolverType::PPP => ctx.sp3_data().unwrap().epoch().collect(),
        };
        let cosm = Cosm::de438();
        let sun_body = Bodies::Sun;
        let earth_j2000 = cosm.frame("EME2000");
        for epoch in epochs {
            let orbit = cosm.celestial_state(
                sun_body.ephem_path(),
                epoch,
                earth_j2000,
                LightTimeCalc::None,
            );
            ret.insert(epoch, (orbit.x_km, orbit.y_km, orbit.z_km));
        }
        ret
    }
    /*
     * Returns Epoch starting and possible ending of Eclipse condition
     */
    fn eclipses(
        ctx: &QcContext,
        sun: &HashMap<Epoch, (f64, f64, f64)>,
    ) -> HashMap<Sv, (Epoch, Option<Epoch>)> {
        let mut ret: HashMap<Sv, (Epoch, Option<Epoch>)> = HashMap::new();
        if let Some(nav) = ctx.navigation_data() {
            for (epoch, (sv, pos_x, pos_y, pos_z)) in nav.sv_position() {}
        }
        ret
    }
    pub fn eclipse_filter_mut(&self, ctx: &mut QcContext) {
        let eclipses = Self::eclipses(ctx, &self.sun);
        for (sv, (start, end)) in eclipses {}
        let filt = Filter::from_str("=gps").unwrap();
        ctx.primary_data_mut().filter_mut(filt);
    }
}
