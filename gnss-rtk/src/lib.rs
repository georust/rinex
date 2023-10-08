use nyx_space::cosmic::eclipse::{eclipse_state, EclipseState};
use nyx_space::cosmic::{Orbit, SPEED_OF_LIGHT};
use nyx_space::md::prelude::{Bodies, LightTimeCalc};
use rinex::navigation::Ephemeris;
use rinex::prelude::{
    //Duration,
    Epoch,
    Sv,
};
use rinex_qc::QcContext;
use std::collections::HashMap;

use hifitime::{Duration, TimeScale, Unit};

extern crate nyx_space as nyx;

use nalgebra::base::{
    DVector,
    MatrixXx4,
    //Vector1,
    //Vector3,
    //Vector4,
};
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

use log::{debug, error, trace, warn};

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

        /*
         * print some infos on latched config
         */
        if self.cfg.modeling.earth_rotation {
            warn!("can't compensate for earth rotation at the moment");
        }
        if self.cfg.modeling.relativistic_clock_corr {
            warn!("relativistic clock corr. is not feasible at the moment");
        }
        if self.solver == SolverType::PPP && self.cfg.min_sv_sunlight_rate.is_some() {
            warn!("eclipse filter is not meaningful when using spp strategy");
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

        trace!("{:?}: {} candidates", t, elected_sv.len());

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

        // apply first set of filters : on OBSERVATION
        //  - no pseudo range: nothing is feasible
        //  - if we're in ppp mode: must be compliant
        //  - if an SNR mask is defined: SNR must be good enough
        elected_sv.retain(|sv| {
            let has_pr = pr
                .iter()
                .filter_map(|(svnn, pr)| if svnn == sv { Some(pr) } else { None })
                .reduce(|pr, _| pr)
                .is_some();

            let mut ppp_ok = !(self.solver == SolverType::PPP);
            if self.solver == SolverType::PPP {
                //TODO: verify PPP compliancy
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

            if !has_pr {
                trace!("{:?}: {} no pseudo range", t, sv);
            }
            if !ppp_ok {
                trace!("{:?}: {} not ppp compliant", t, sv);
            }
            if !snr_ok {
                trace!("{:?}: {} snr below criteria", t, sv);
            }

            has_pr && snr_ok & ppp_ok
        });

        // make sure we still have enough SV
        if elected_sv.len() < 4 {
            debug!("{:?}: not enough vehicles elected", t);
            self.nth_epoch += 1;
            return Err(SolverError::LessThan4Sv(t));
        }

        debug!("{:?}: {} elected sv", t, elected_sv.len());

        let mut sv_data: HashMap<Sv, (f64, f64, f64, f64, Duration)> = HashMap::new();

        // 3: sv position evaluation
        for sv in &elected_sv {
            // retrieve pr for this SV @ t
            let pr = pr
                .iter()
                .filter_map(|(svnn, pr)| if svnn == sv { Some(*pr) } else { None })
                .reduce(|pr, _| pr)
                .unwrap(); // can't fail at this point

            let ts = sv.timescale().unwrap(); // can't fail at this point ?

            let nav = ctx.navigation_data().unwrap(); // can't fail at this point ?

            let ephemeris = nav.sv_ephemeris(*sv, t);
            if ephemeris.is_none() {
                error!("{:?} : {} no valid ephemeris", t, sv);
                continue;
            }

            let (toe, eph) = ephemeris.unwrap();
            let clock_bias = eph.sv_clock();
            let (t_tx, dt_sat) =
                Self::sv_transmission_time(t, *sv, toe, pr, eph, modeling, clock_bias, ts);

            if modeling.earth_rotation {
                //TODO
                // dt = || rsat - rcvr0 || /c
                // rsat = R3 * we * dt * rsat
                // we = 7.2921151467 E-5
            }

            if modeling.relativistic_clock_corr {
                //TODO
                let e = 1.204112719279E-2;
                let sqrt_a = 5.153704689026E3;
                let sqrt_mu = (3986004.418E8_f64).sqrt();
                //let dt = -2.0_f64 * sqrt_a * sqrt_mu / SPEED_OF_LIGHT / SPEED_OF_LIGHT * e * elev.sin();
            }

            // interpolate
            let pos: Option<(f64, f64, f64)> = match ctx.sp3_data() {
                Some(sp3) => {
                    /*
                     * SP3 always prefered
                     */
                    let pos = sp3.sv_position_interpolate(*sv, t_tx, interp_order);
                    if let Some(pos) = pos {
                        Some(pos)
                    } else {
                        /* try to fall back to ephemeris nav */
                        nav.sv_position_interpolate(*sv, t_tx, interp_order)
                    }
                },
                _ => nav.sv_position_interpolate(*sv, t_tx, interp_order),
            };

            if pos.is_none() {
                trace!("{:?} : {} interpolation failed", t, sv);
                continue;
            }

            let (x_km, y_km, z_km) = pos.unwrap();

            // Elevation filter
            if let Some(min_elev) = self.cfg.min_sv_elev {
                let (e, _) = Ephemeris::elevation_azimuth(
                    (x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3),
                    pos0.into(),
                );
                if e < min_elev {
                    trace!("{:?} : {} elev below mask", t, sv);
                    continue;
                }
            }

            // Eclipse filter
            if let Some(min_rate) = self.cfg.min_sv_sunlight_rate {
                let state = self.eclipse_state(x_km, y_km, z_km, t_tx);
                let eclipsed = match state {
                    EclipseState::Umbra => true,
                    EclipseState::Visibilis => false,
                    EclipseState::Penumbra(r) => r < min_rate,
                };
                if eclipsed {
                    debug!("{:?} : dropping eclipsed {}", t, sv);
                } else {
                    sv_data.insert(*sv, (x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3, pr, dt_sat));
                }
            } else {
                sv_data.insert(*sv, (x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3, pr, dt_sat));
            }
        }

        // 6: form matrix
        let mut y = DVector::<f64>::zeros(elected_sv.len());
        let mut g = MatrixXx4::<f64>::zeros(elected_sv.len());

        if sv_data.iter().count() < 4 {
            error!("{:?} : not enough sv to resolve", t);
            self.nth_epoch += 1;
            return Err(SolverError::LessThan4Sv(t));
        }

        for (index, (sv, data)) in sv_data.iter().enumerate() {
            let pr = data.3;
            let dt_sat = data.4.to_seconds();
            let (sv_x, sv_y, sv_z) = (data.0, data.1, data.2);

            let rho = ((sv_x - x0).powi(2) + (sv_y - y0).powi(2) + (sv_z - z0).powi(2)).sqrt();

            //TODO
            let mut models = -SPEED_OF_LIGHT * dt_sat;
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

            y[index] = pr - rho - models;

            g[(index, 0)] = (x0 - sv_x) / rho;
            g[(index, 1)] = (y0 - sv_y) / rho;
            g[(index, 2)] = (z0 - sv_z) / rho;
            g[(index, 3)] = 1.0_f64;
        }

        // 7: resolve
        //trace!("y: {} | g: {}", y, g);
        let estimate = SolverEstimate::new(g, y);
        self.nth_epoch += 1;

        if estimate.is_none() {
            return Err(SolverError::SolvingError(t));
        } else {
            Ok((t, estimate.unwrap()))
        }
    }
    /*
     * Evalutes Sv position
     */
    fn sv_transmission_time(
        t: Epoch,
        sv: Sv,
        toe: Epoch,
        pr: f64,
        eph: &Ephemeris,
        m: Modeling,
        clock_bias: (f64, f64, f64),
        ts: TimeScale,
    ) -> (Epoch, Duration) {
        let seconds_ts = t.to_duration().to_seconds();

        let dt_tx = seconds_ts - pr / SPEED_OF_LIGHT;
        let mut e_tx = Epoch::from_duration(dt_tx * Unit::Second, t.time_scale);
        let mut dt_sat = Duration::default();

        if m.sv_clock_bias {
            dt_sat = Ephemeris::sv_clock_corr(sv, clock_bias, t, toe);
            debug!("{:?}: {} dt_sat  {}", t, sv, dt_sat);
            e_tx -= dt_sat;
        }

        if m.sv_total_group_delay {
            if let Some(tgd) = eph.tgd() {
                let tgd = tgd * Unit::Second;
                debug!("{:?}: {} tgd      {}", t, sv, tgd);
                e_tx -= tgd;
            }
        }

        debug!("{:?}: {} t_tx      {:?}", t, sv, e_tx);

        /*
         * physical verification on result
         */
        let dt = (t - e_tx).to_seconds();
        assert!(dt > 0.0, "t_tx can't physically be after t_rx..!");
        assert!(
            dt < 1.0,
            "|t - t_tx| < 1s is physically impossible (signal propagation..)"
        );

        (e_tx, dt_sat)
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
