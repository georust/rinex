use nyx_space::cosmic::SPEED_OF_LIGHT;
use nyx_space::md::prelude::{Bodies, Cosm, LightTimeCalc};
use rinex::prelude::{Duration, Epoch, Sv};
use rinex_qc::QcContext;
use std::collections::HashMap;
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
    /// Whether this solver is initiated (ready to iterate) or not
    initiated: bool,
    t_tx: Vec<(Sv, Epoch, Epoch)>,
    /// Type of solver implemented
    pub solver: SolverType,
    /// Current position estimate
    pub estimated_pos: (f64, f64, f64),
    /// Current Epoch estimate
    pub estimated_time: Epoch,
}

impl Solver {
    pub fn from(context: &QcContext) -> Result<Self, Error> {
        let solver = SolverType::from(context)?;
        Ok(Self {
            solver,
            t_tx: vec![],
            initiated: false,
            opts: SolverOpts::default(),
            sun: Self::sun_vector3d(context, solver),
            estimated_pos: (0.0_f64, 0.0_f64, 0.0_f64),
            estimated_time: Epoch::default(),
        })
    }
    pub fn run(&mut self, ctx: &mut QcContext) -> ((f64, f64, f64), Epoch) {
        if !self.initiated {
            // 1: eclipse filter
            self.sun = Self::sun_vector3d(ctx, self.solver);
            self.eclipse_filter(ctx);
            // 2: interpolate if needed
            if !ctx.interpolated {
                ctx.sv_orbit_interpolation();
            }
            // 3: t_tx
            self.t_tx = Self::sv_transmission_time(ctx).collect();
            self.initiated = true;
            trace!("{} solver initiated", self.solver);
        }
        (self.estimated_pos, self.estimated_time)
    }
    /*
     * Returns Sv clock data from given context
     */
    fn sv_clock(ctx: &QcContext) -> Box<dyn Iterator<Item = (Epoch, Sv, f64)> + '_> {
        match ctx.sp3_data() {
            Some(sp3) => Box::new(sp3.sv_clock()),
            _ => Box::new(
                ctx.navigation_data()
                    .unwrap()
                    .sv_clock()
                    .map(|(e, sv, (clk, _, _))| (e, sv, clk)),
            ),
        }
    }
    /*
     * Iterators over each individual Sv T_tx transmission time
     * in the form (Sv, T_tx, T_rx) where T are expressed as hifitime::Epoch
     */
    fn sv_transmission_time(ctx: &QcContext) -> Box<dyn Iterator<Item = (Sv, Epoch, Epoch)> + '_> {
        Box::new(Self::sv_clock(ctx).filter_map(|(t_rx, sv, sv_clk)| {
            // need one pseudo range observation
            // for this Sv @ this epoch
            let mut pr = ctx
                .primary_data()
                .pseudo_range()
                .filter_map(|((e, flag), svnn, _, value)| {
                    if e == t_rx && flag.is_ok() && svnn == sv {
                        Some(value)
                    } else {
                        None
                    }
                })
                .take(1); // dont care about signals : just need one
            if let Some(pr) = pr.next() {
                let t_tx = t_rx.to_duration().to_seconds() - pr / SPEED_OF_LIGHT - sv_clk;
                let t_tx = Epoch::from_duration(Duration::from_seconds(t_tx), t_rx.time_scale);
                debug!("t_rx: {} | t_tx {}", t_rx, t_tx);
                Some((sv, t_tx, t_rx))
            } else {
                debug!("{} @{} - missing pseudo range observation", sv, t_rx);
                None
            }
        }))
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
        let eme_j2000 = cosm.frame("EME2000");
        for epoch in epochs {
            let orbit =
                cosm.celestial_state(sun_body.ephem_path(), epoch, eme_j2000, LightTimeCalc::None);
            ret.insert(epoch, (orbit.x_km, orbit.y_km, orbit.z_km));
        }
        ret
    }
    /*
     * Computes celestial angle condition
     */
    fn eclipsed(&self, x: f64, y: f64, z: f64, e: Epoch) -> bool {
        let mean_equatorial = 6378137.0_f64;
        let (sun_x_km, sun_y_km, sun_z_km) = self.sun.get(&e).unwrap();
        let (sun_x, sun_y, sun_z) = (sun_x_km / 1000.0, sun_y_km / 1000.0, sun_z_km / 1000.0);
        let dot = sun_x * x + sun_y * y + sun_z * z;
        let norm_sv = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
        let norm_sun = (sun_x.powi(2) + sun_y.powi(2) + sun_z.powi(2)).sqrt();
        let cos_phi = dot / norm_sv / norm_sun;
        let azim = norm_sv * (1.0 - cos_phi.powi(2)).sqrt();
        cos_phi < 0.0 && azim < mean_equatorial
    }
    /*
     * Returns Eclipse Start/End Epoch and related vehicle identity
     */
    fn eclipses(
        &self,
        ctx: &QcContext,
        min_dt: Duration,
    ) -> HashMap<Sv, (Option<Epoch>, Option<Epoch>)> {
        let mut rising: HashMap<Sv, Epoch> = HashMap::new();
        let mut ret: HashMap<Sv, (Option<Epoch>, Option<Epoch>)> = HashMap::new();

        for (epoch, sv, (x, y, z)) in ctx.sv_position() {
            let eclipsed = self.eclipsed(x, y, z, epoch);
            if eclipsed && ret.get_mut(&sv).is_none() {
                // start of eclipse
                ret.insert(sv, (Some(epoch), None));
                debug!("{}: start of eclipse @{}", sv, epoch);
            } else if !eclipsed {
                if ret.get_mut(&sv).is_some() {
                    if rising.get_mut(&sv).is_none() {
                        // end of eclipse starting
                        rising.insert(sv, epoch);
                    } else {
                        let start = rising.get(&sv).unwrap();
                        if let Some((beginning, None)) = ret.get(&sv) {
                            if epoch - *start > min_dt {
                                // end of eclipse
                                debug!("{}: end of eclipse @{}", sv, epoch);
                                ret.insert(sv, (*beginning, Some(epoch)));
                            }
                        }
                    }
                } else {
                    if rising.get_mut(&sv).is_none() {
                        // end of eclipse starting
                        rising.insert(sv, epoch);
                    } else {
                        let start = rising.get(&sv).unwrap();
                        if let Some((beginning, None)) = ret.get(&sv) {
                            if epoch - *start > min_dt {
                                debug!("{}: end of eclipse @{}", sv, epoch);
                                ret.insert(sv, (*beginning, Some(epoch)));
                            }
                        }
                    }
                }
            }
        }
        ret
    }
    /*
     * Remove all Sv that are in Eclipse::umbra condition
     */
    fn eclipse_filter(&self, ctx: &mut QcContext) {
        let eclipses = self.eclipses(ctx, Duration::from_seconds(30.0 * 60.0));
        for (sv, (start, end)) in eclipses {
            // design interval filter
            let record = ctx
                .primary_data_mut()
                .record
                .as_mut_obs()
                .expect("primary file should be OBS");
            match start {
                Some(start) => match end {
                    Some(end) => {
                        record.retain(|(e, _), (_, vehicles)| {
                            vehicles.retain(|svnn, _| {
                                if *svnn == sv {
                                    *e < start || *e > end
                                } else {
                                    true
                                }
                            });
                            vehicles.len() > 0
                        });
                    },
                    _ => {
                        record.retain(|(e, _), (_, vehicles)| {
                            vehicles.retain(|svnn, _| if *svnn == sv { *e < start } else { true });
                            vehicles.len() > 0
                        });
                    },
                },
                _ => match end {
                    Some(end) => {
                        record.retain(|(e, _), (_, vehicles)| {
                            vehicles.retain(|svnn, _| if *svnn == sv { *e > end } else { true });
                            vehicles.len() > 0
                        });
                    },
                    _ => {},
                },
            }
        }
    }
}
