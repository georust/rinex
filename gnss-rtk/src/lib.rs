#![doc = include_str!("../README.md")]
#![cfg_attr(docrs, feature(doc_cfg))]

use nyx_space::cosmic::eclipse::{eclipse_state, EclipseState};
use nyx_space::cosmic::{Orbit, SPEED_OF_LIGHT};
use nyx_space::md::prelude::{Bodies, Frame, LightTimeCalc};

use hifitime::Epoch;

extern crate gnss_rs as gnss;
extern crate nyx_space as nyx;

use gnss::prelude::SV;

use nalgebra::base::{
    DVector,
    MatrixXx4,
    //Vector1,
    //Vector3,
    //Vector4,
};
use nyx::md::prelude::{Arc, Cosm};

mod cfg;
mod model;
mod vector;

mod apriori;
mod candidate;
mod estimate;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// SPP : code based and approximated models
    /// aiming a metric resolution.
    #[default]
    SPP,
    // /// PPP : phase + code based, the ultimate solver
    // /// aiming a millimetric resolution.
    // PPP,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SPP => write!(fmt, "SPP"),
            // Self::PPP => write!(fmt, "PPP"),
        }
    }
}

pub mod prelude {
    pub use crate::cfg::Config;
    pub use crate::estimate::Estimate;
    pub use crate::model::Modeling;
    pub use crate::Error;
    pub use crate::Mode;
    pub use crate::Solver;
    pub use hifitime::{Duration, Epoch, TimeScale};
}

use cfg::Config;
use estimate::Estimate;
use model::{
    Modelization,
    Models,
    // Modeling,
};
use vector::Vector3D;

use apriori::AprioriPosition;
use candidate::Candidate;
use model::TropoComponents;

use log::{debug, error, trace, warn};

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("not enough vehicles elected @{0}")]
    LessThan4Sv(Epoch),
    #[error("failed to invert navigation matrix @{0}")]
    SolvingError(Epoch),
    #[error("undefined apriori position")]
    UndefinedAprioriPosition,
    #[error("at least one pseudo range observation is mandatory")]
    NeedsAtLeastOnePseudoRange,
}

/// Interpolation result that your data interpolator should return
/// For Solver.resolve() to truly complete.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct InterpolationResult {
    /// Position in the sky
    sky_pos: Vector3D,
    /// Optional elevation compared to reference position and horizon
    elevation: Option<f64>,
    /// Optional azimuth compared to reference position and magnetic North
    azimuth: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Solver {
    /// Solver parametrization
    pub cfg: Config,
    /// Type of solver implemented
    pub mode: Mode,
    /// apriori position
    pub apriori: AprioriPosition,
    /// Interpolation method: must always resolve InterpolationResults
    /// correctly at given Epoch with desired interpolation order, for the solver to resolve.
    pub interpolator: fn(Epoch, SV, usize) -> Option<InterpolationResult>,
    /// Custom tropospheric delay components provider.
    /// If you want to implement the tropospheric delay compensation yourself, or have a better source of such data, use this. Otherwise,
    /// the solver will implement its own compensator
    pub tropo_components: fn(Epoch, f64, f64) -> Option<TropoComponents>,
    /// cosmic model
    cosmic: Arc<Cosm>,
    /// Earth frame
    earth_frame: Frame,
    /// Sun frame
    sun_frame: Frame,
    /// modelization memory storage
    models: Models,
}

impl Solver {
    pub fn new(
        mode: Mode,
        apriori: AprioriPosition,
        cfg: &Config,
        interpolator: fn(Epoch, SV, usize) -> Option<InterpolationResult>,
        tropo_components: fn(Epoch, f64, f64) -> Option<TropoComponents>,
    ) -> Result<Self, Error> {
        let cosmic = Cosm::de438();
        let sun_frame = cosmic.frame("Sun J2000");
        let earth_frame = cosmic.frame("EME2000");

        /*
         * print some infos on latched config
         */
        if cfg.modeling.iono_delay {
            warn!("can't compensate for ionospheric delay at the moment");
        }

        if cfg.modeling.earth_rotation {
            warn!("can't compensate for earth rotation at the moment");
        }

        if cfg.modeling.relativistic_clock_corr {
            warn!("relativistic clock corr. is not feasible at the moment");
        }

        if mode == Mode::SPP && cfg.min_sv_sunlight_rate.is_some() {
            warn!("eclipse filter is not meaningful when using spp strategy");
        }

        Ok(Self {
            mode,
            cosmic,
            sun_frame,
            earth_frame,
            apriori,
            interpolator,
            tropo_components,
            cfg: cfg.clone(),
            models: Models::with_capacity(cfg.max_sv),
        })
    }
    /// Candidates election process, you can either call yourself this method
    /// externally prior a Self.run(), or use "pre_selected: false" in Solver.run()
    /// or use "pre_selected: true" with your own selection method prior using Solver.run().
    pub fn elect_candidates<'a>(
        t: Epoch,
        pool: Vec<Candidate<'a>>,
        mode: Mode,
        cfg: &'a Config,
    ) -> Vec<Candidate<'a>> {
        let mut p = pool.clone();
        p.iter()
            .filter_map(|c| {
                let mode_compliant = match mode {
                    Mode::SPP => true,
                    // Mode::PPP => false, // TODO
                };
                let snr_ok = match cfg.min_sv_snr {
                    Some(snr) => {
                        let ok = c.snr > snr;
                        if !ok {
                            trace!("{:?} : {} snr below criteria", c.t, c.sv);
                        }
                        ok
                    },
                    _ => true,
                };
                if mode_compliant && snr_ok {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    /// Run position solving algorithm, using predefined strategy.
    pub fn run(&mut self, t: Epoch, pool: Vec<Candidate>) -> Result<(Epoch, Estimate), Error> {
        let (x0, y0, z0) = (
            self.apriori.ecef.x,
            self.apriori.ecef.y,
            self.apriori.ecef.z,
        );
        let (lat_ddeg, lon_ddeg, altitude_above_sea_m) = (
            self.apriori.geodetic.x,
            self.apriori.geodetic.y,
            self.apriori.geodetic.z,
        );

        let modeling = self.cfg.modeling;
        let interp_order = self.cfg.interp_order;

        let pool = Self::elect_candidates(t, pool, self.mode, &self.cfg);

        /* interpolate positions */
        let mut pool: Vec<Candidate> = pool
            .iter()
            .filter_map(|c| {
                let mut t_tx = c.transmission_time(&self.cfg).ok()?;

                // TODO : complete this equation please
                if self.cfg.modeling.relativistic_clock_corr {
                    let _e = 1.204112719279E-2;
                    let _sqrt_a = 5.153704689026E3;
                    let _sqrt_mu = (3986004.418E8_f64).sqrt();
                    //let dt = -2.0_f64 * sqrt_a * sqrt_mu / SPEED_OF_LIGHT / SPEED_OF_LIGHT * e * elev.sin();
                    // t_tx -=
                }

                // TODO : requires instantaneous speed
                if self.cfg.modeling.earth_rotation {
                    // dt = || rsat - rcvr0 || /c
                    // rsat = R3 * we * dt * rsat
                    // we = 7.2921151467 E-5
                }

                if let Some(interpolated) = (self.interpolator)(t_tx, c.sv, self.cfg.interp_order) {
                    let mut c = c.clone();
                    c.state = Some(interpolated.sky_pos);
                    c.elevation = interpolated.elevation;
                    c.azimuth = interpolated.azimuth;
                    Some(c)
                } else {
                    None
                }
            })
            .collect();

        /* apply elevation filter (if any) */
        if let Some(min_elev) = self.cfg.min_sv_elev {
            for idx in 0..pool.len() {
                let elevation = pool[idx].elevation.unwrap(); // infaillible
                if elevation < min_elev {
                    let _ = pool.swap_remove(idx);
                }
            }
        }

        /* apply eclipse filter (if need be) */
        if let Some(min_rate) = self.cfg.min_sv_sunlight_rate {
            for idx in 0..pool.len() {
                let state = pool[idx].state.unwrap(); // infaillible
                let (x, y, z) = (state.x, state.y, state.z);
                let orbit = Orbit {
                    x_km: x / 1000.0,
                    y_km: y / 1000.0,
                    z_km: z / 1000.0,
                    vx_km_s: 0.0_f64, // TODO ?
                    vy_km_s: 0.0_f64, // TODO ?
                    vz_km_s: 0.0_f64, // TODO ?
                    epoch: pool[idx].t,
                    frame: self.earth_frame,
                    stm: None,
                };
                let state = eclipse_state(&orbit, self.sun_frame, self.earth_frame, &self.cosmic);
                let eclipsed = match state {
                    EclipseState::Umbra => true,
                    EclipseState::Visibilis => false,
                    EclipseState::Penumbra(r) => r < min_rate,
                };
                if eclipsed {
                    debug!("{:?} : dropping eclipsed {}", pool[idx].t, pool[idx].sv);
                    let _ = pool.swap_remove(idx);
                }
            }
        }

        /* make sure we still have enough SV */
        let nb_candidates = pool.len();
        if nb_candidates < 4 {
            debug!(
                "{:?}: {} sv is not enough to generate a solution",
                t, nb_candidates
            );
            return Err(Error::LessThan4Sv(t));
        } else {
            debug!("{:?}: {} elected sv", t, nb_candidates);
        }

        /* modelization */
        self.models.modelize(
            t,
            pool.iter().map(|c| (c.sv, c.elevation.unwrap())).collect(),
            lat_ddeg,
            altitude_above_sea_m,
            &self.cfg,
            self.tropo_components,
        );

        /* form matrix */
        let mut y = DVector::<f64>::zeros(nb_candidates);
        let mut g = MatrixXx4::<f64>::zeros(nb_candidates);

        for (index, c) in pool.iter().enumerate() {
            let sv = c.sv;
            let pr = c.pseudo_range();
            let dt_sat = c.clock_corr.to_seconds();
            let state = c.state.unwrap(); // infaillible
            let (sv_x, sv_y, sv_z) = (state.x, state.y, state.z);

            // let code = data.3;

            let rho = ((sv_x - x0).powi(2) + (sv_y - y0).powi(2) + (sv_z - z0).powi(2)).sqrt();

            let models = -SPEED_OF_LIGHT * dt_sat + self.models.sum_up(sv);

            y[index] = pr - rho - models;

            /*
             * accurate time delay compensation (if any)
             */
            // if let Some(int_delay) = self.cfg.internal_delay.get(code) {
            //     y[index] -= int_delay * SPEED_OF_LIGHT;
            // }

            // if let Some(timeref_delay) = self.cfg.time_ref_delay {
            //     y[index] += timeref_delay * SPEED_OF_LIGHT;
            // }

            g[(index, 0)] = (x0 - sv_x) / rho;
            g[(index, 1)] = (y0 - sv_y) / rho;
            g[(index, 2)] = (z0 - sv_z) / rho;
            g[(index, 3)] = 1.0_f64;
        }

        // 7: resolve
        //trace!("y: {} | g: {}", y, g);
        let estimate = Estimate::new(g, y);

        if estimate.is_none() {
            Err(Error::SolvingError(t))
        } else {
            Ok((t, estimate.unwrap()))
        }
    }
    /*
     * Evaluates Sun/Earth vector, <!> expressed in Km <!>
     * for all SV NAV Epochs in provided context
     */
    fn sun_earth_vector(&mut self, t: Epoch) -> Vector3D {
        let sun_body = Bodies::Sun;
        let orbit = self.cosmic.celestial_state(
            sun_body.ephem_path(),
            t,
            self.earth_frame,
            LightTimeCalc::None,
        );
        Vector3D {
            x: orbit.x_km * 1000.0,
            y: orbit.y_km * 1000.0,
            z: orbit.z_km * 1000.0,
        }
    }
}
