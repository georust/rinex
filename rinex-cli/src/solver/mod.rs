use nyx_space::cosmic::eclipse::{eclipse_state, EclipseState};
use nyx_space::cosmic::{Orbit, SPEED_OF_LIGHT};
use nyx_space::md::prelude::{Arc, Bodies, Cosm, LightTimeCalc};
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
    /// SPP : code based and approximated models
    /// aiming a metric resolution.
    #[default]
    SPP,
    /// PPP : phase + code based, the ultimate solver
    /// aiming a millimetric resolution.
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

#[derive(Debug)]
pub(crate) struct Solver {
    /// Cosmic model
    cosmic: Arc<Cosm>,
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
            cosmic: Cosm::de438(),
            solver,
            t_tx: vec![],
            initiated: false,
            opts: SolverOpts::default(),
            sun: HashMap::new(),
            estimated_pos: (0.0_f64, 0.0_f64, 0.0_f64),
            estimated_time: Epoch::default(),
        })
    }
    pub fn run(&mut self, ctx: &mut QcContext) -> ((f64, f64, f64), Epoch) {
        if !self.initiated {
            // 0: NB: only "complete" Epochs are preserved from now on
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

            // 1: eclipse filter
            self.eval_sun_vector3d(ctx);
            self.eclipse_filter(ctx);

            // 2: interpolate: if need be
            if !ctx.interpolated {
                trace!("orbit interpolation..");
                ctx.orbit_interpolation(7, None);
                //TODO could be nice to have some kind of timing/perf evaluation here
                //     and also total number of required interpolations
            }

            // 3: t_tx evaluation
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
                .take(1); // dont need other signals : just one
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
    fn eval_sun_vector3d(&mut self, ctx: &QcContext) {
        trace!("Earth / Sun vector evaluation..");
        let mut ret: HashMap<Epoch, (f64, f64, f64)> = HashMap::new();
        let epochs: Vec<Epoch> = match self.solver {
            SolverType::SPP => ctx.navigation_data().unwrap().epoch().collect(),
            SolverType::PPP => ctx.sp3_data().unwrap().epoch().collect(),
        };
        let sun_body = Bodies::Sun;
        let eme_j2000 = self.cosmic.frame("EME2000");
        for epoch in epochs {
            let orbit = self.cosmic.celestial_state(
                sun_body.ephem_path(),
                epoch,
                eme_j2000,
                LightTimeCalc::None,
            );
            self.sun.insert(epoch, (orbit.x_km, orbit.y_km, orbit.z_km));
        }
    }
    /*
     * Computes celestial angle condition
     */
    fn eclipsed(&self, x_km: f64, y_km: f64, z_km: f64, epoch: Epoch) -> bool {
        let mean_equatorial = 6378137.0_f64;
        let sun_frame = self.cosmic.frame("ICRF");
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
        trace!("applying eclipse filter..");
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
