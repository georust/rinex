use nyx_space::cosmic::eclipse::{eclipse_state, EclipseState};
use nyx_space::cosmic::{Orbit, SPEED_OF_LIGHT as C};
use nyx_space::md::prelude::{Bodies, LightTimeCalc};
use rinex::carrier::Carrier;
use rinex::navigation::Ephemeris;
use rinex::prelude::{
    //Duration,
    Epoch,
    Observable,
    RnxContext,
};
use std::collections::HashMap;

use hifitime::{Duration, Unit};

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
mod apriori;
mod estimate;
mod candidate;

pub mod prelude {
    pub use crate::cfg::RTKConfig;
    pub use crate::cfg::SolverMode;
    pub use crate::estimate::SolverEstimate;
    pub use crate::model::Modeling;
    pub use crate::RTKContext;
    pub use crate::Solver;
    pub use crate::SolverError;
    pub use crate::SolverMode;
}

use cfg::RTKConfig;
use estimate::SolverEstimate;
use model::{Modeling, Modelization, Models};

use log::{debug, error, trace, warn};

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum SolverError {
    #[error("not enough vehicles elected @{0}")]
    LessThan4Sv(Epoch),
    #[error("failed to invert navigation matrix @{0}")]
    SolvingError(Epoch),
    #[error("undefined apriori position")]
    UndefinedAprioriPosition,
}

/// Interpolation result that your data interpolator should return
/// For Solver.resolve() to truly complete.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct InterpolationResult {
    /// Position in the sky 
    sky_pos: Vector3D,
    /// Optional elevation compared to reference position and horizon
    elev: Option<f64>,
    /// Optional azimuth compared to reference position and magnetic North
    azimuth: Option<f64>, 
}

#[derive(Debug, Copy, Clone)]
pub struct Vector3D {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone)]
pub struct Solver {
    /// Solver parametrization
    pub cfg: RTKConfig,
    /// Type of solver implemented
    pub solver: SolverMode,
    /// cosmic model
    cosmic: Arc<Cosm>,
    /// Earth frame
    earth_frame: Frame,
    /// Sun frame
    sun_frame: Frame,
    /// current epoch
    nth_epoch: usize,
    /// modelization memory storage
    models: Models,
    /// apriori position in ECEF
    pub apriori_position: Vector3D,
    /// apriori geodetic position (lat [ddeg], lon [ddeg], altitude (above sea) [m])
    pub apriori_geodetic_position: Vector3D, 
    /// Interpolation method: must always resolve InterpolationResults 
    /// correctly at given Epoch with desired interpolation order, for the solver to resolve.
    pub interpolator : fn(SV, Epoch, usize) -> Option<InterpolationResult>,
    /// Clock correction method: must resolve a Duration correctly
    /// at given Epoch for the solver to proceed, except if cfg.sv_clock_bias
    /// is turned off.
    pub clock_corr : fn(Epoch) -> Option<Duration>,
}

impl Solver {
    pub fn new(mode: SolverMode, apriori: AprioriPosition, cfg: &RTKConfig) -> Result<Self, SolverError> {
        let cosmic = Cosm::de438();
        let sun_frame = cosmic.frame("Sun J2000");
        let earth_frame = cosmic.frame("EME2000");
        let cfg = RTKConfig::default(solver);
        let solver = SolverMode::from(context)?;

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
        
        if mode == SolverMode::PPP && cfg.min_sv_sunlight_rate.is_some() {
            warn!("eclipse filter is not meaningful when using spp strategy");
        }
        
        let (apriori_position, apriori_geodetic_position) = Self::apriori_position(cfg)
            .ok_or(SolverError::UndefinedAprioriPosition)?;

        Ok(Self {
            cosmic,
            sun_frame,
            earth_frame,
            apriori,
            solver,
            cfg: cfg.clone(),
            models: Models::with_capacity(cfg.max_sv),
        })
    }
    /// Candidates election process, you can either call yourself this method
    /// externally prior a Self.run(), or use "pre_selected: false" in Solver.run()
    /// or use "pre_selected: true" with your own selection method prior using Solver.run().
    pub fn election_process(t: Epoch, pool: Vec<Candidate>, mode: SolverMode, cfg: &RTKConfig) -> Vec<Candidate> {
        let mut p = pool.clone();
        p.iter()
            .filter_map(|c| {
                let mode_compliant = match mode {
                    SolverMode::SPP => true,
                    SolverMode::PPP => false, // TODO
                };
                let snr_ok = match cfg.min_sv_snr {
                    Some(snr) => {
                        let ok = c.snr > snr;
                        if !ok {
                            trace!("{:?} : snr below criteria", t, sv);
                        }
                    },
                    _ => true,
                };
                mode_compliant && snr_ok
            })
    }
    /// Run position solving algorithm, using predefined strategy.
    /// "pool": List of candidates. 
    /// "pre_selected": set to true in case you don't want to run the election process.
    /// The election process will select vehicles among all candidates that fit the predefined
    /// requirements (see solver configuration).
    pub fn run(&mut self, t: Epoch, pool: Vec<Candidate>, pre_selected: bool) -> Result<(Epoch, SolverEstimate), SolverError> {

        let (x0, y0, z0) = self.apriori.ecef;
        let (lat_ddeg, lon_ddeg, altitude_above_sea_m) = self.apriori.geodetic;

        let modeling = self.cfg.modeling;
        let interp_order = self.cfg.interp_order;

        let pool = match pre_selected {
            false => Self::election_process(self.mode, self.cfg, pool)?,
            true => pool.clone(),
        };

        /* interpolate positions */
        let mut pool: Vec<Candidate> = pool.iter()
            .filter_map(|c| {
                let mut t_tx = c.transmission_time()?;
                
                // TODO : complete this equation please
                if self.modeling.relativistic_clock_corr {
                    let _e = 1.204112719279E-2;
                    let _sqrt_a = 5.153704689026E3;
                    let _sqrt_mu = (3986004.418E8_f64).sqrt();
                    //let dt = -2.0_f64 * sqrt_a * sqrt_mu / SPEED_OF_LIGHT / SPEED_OF_LIGHT * e * elev.sin();
                    // t_tx -=
                }

                // TODO : requires instantaneous speed
                if self.modeling.earth_rotation {
                    // dt = || rsat - rcvr0 || /c
                    // rsat = R3 * we * dt * rsat
                    // we = 7.2921151467 E-5
                }

                let interpolated = self.interpolator(t_tx)?;
                c.state = Some(interpolated.sky_pos);
                c.elevation = Some(interpolated.elevation);
                c.azimuth = Some(interpolated.azimuth);
               
                // apply elev filter, if any
                if let Some(min_elev) = self.cfg.min_sv_elev {
                    if interpolated.elevation < min_elev {
                        trace!("{:?} : {} elev below mask", c.t, c.sv);
                    }
                } else {
                    Some(c)
                }
            })
            .collect();

        /* apply elevation filter (if any) */
        if let Some(min_elev) = self.cfg.min_sv_elev {
            for idx in 0..pool.len() {
                let elevation = pool[index].elevation.unwrap(); // infaillible
                if elevation < min_elev {
                    let _ = pool.swap_remove(idx);
                }
            }
        }

        /* apply eclipse filter (if need be) */
        if let Some(min_rate) = self.cfg.min_sv_sunlight_rate {
            for idx in 0..pool.len() {
                let (x, y, z) = pool[idx].state;
                let orbit = Orbit {
                    x_km: x /1000.0,
                    y_km: y /1000.0,
                    z_km: z /1000.0,
                    vx_km_s: 0.0_f64, // TODO ?
                    vy_km_s: 0.0_f64, // TODO ?
                    vz_km_s: 0.0_f64, // TODO ?
                };
                let state = eclipse_state(
                    orbit, 
                    self.sun_frame, 
                    self.earth_frame,
                    self.cosmic,
                );
                let eclipsed = match state {
                    EclipseState::Umbra => true,
                    EclipseState::Visibilis => false,
                    EclipseState::Penumbra(r) => r < min_rate,
                };
                if eclipsed {
                    debug!("{:?} : dropping eclipsed {}", t, sv);
                    let _ = pool.swap_remove(idx);
                }
            }
        }
        
        /* make sure we still have enough SV */
        let nb_candidates = pool.len();
        if nb_candidates < 4 {
            debug!("{:?}: {} sv is not enough to generate a solution", t, nb_candidates);
            return Err(SolverError::LessThan4Sv(t));
        } else {
            debug!("{:?}: {} elected sv", t, nb_candidates);
        }
        
        /* modelization */
        self.models.modelize(
            t,
            pool 
                .iter()
                .map(|c| (c.sv, c.elevation.unwrap()))
                .collect(),
            lat_ddeg,
            altitude_above_sea_m,
            &self.cfg,
        );

        // 7: form matrix
        let mut y = DVector::<f64>::zeros(elected_sv.len());
        let mut g = MatrixXx4::<f64>::zeros(elected_sv.len());

        for (index, (sv, data)) in sv_data.iter().enumerate() {
            let (sv_x, sv_y, sv_z) = (data.0, data.1, data.2);
            let code = data.3;
            let pr = data.4;
            let dt_sat = data.5.to_seconds();

            let rho = ((sv_x - x0).powi(2) + (sv_y - y0).powi(2) + (sv_z - z0).powi(2)).sqrt();

            let models = -SPEED_OF_LIGHT * dt_sat + self.models.sum_up(*sv);

            y[index] = pr - rho - models;

            /*
             * accurate time delay compensation (if any)
             */
            if let Some(int_delay) = self.cfg.internal_delay.get(code) {
                y[index] -= SPEED_OF_LIGHT * int_delay;
            }

            if let Some(timeref_delay) = self.cfg.time_ref_delay {
                y[index] += SPEED_OF_LIGHT * timeref_delay;
            }

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
            Err(SolverError::SolvingError(t))
        } else {
            Ok((t, estimate.unwrap()))
        }
    }
    /*
     * Evalutes SV position
     */
    fn sv_transmission_time(
        t: Epoch,
        sv: SV,
        toe: Epoch,
        pr: f64,
        eph: &Ephemeris,
        m: Modeling,
        clock_bias: (f64, f64, f64),
    ) -> (Epoch, Duration) {

        let mut dt_sat = Duration::default();

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
    fn sun_earth_vector(&mut self, t: Epoch) -> Vector3D {
        let sun_body = Bodies::Sun;
        let orbit =
            self.cosmic
                .celestial_state(sun_body.ephem_path(), t, self.earth_frame, LightTimeCalc::None);
        (orbit.x_km, orbit.y_km, orbit.z_km)
    }
    /*
     * Returns all SV at "t"
     */
    fn sv_at_epoch(ctx: &RnxContext, t: Epoch) -> Option<Vec<SV>> {
        ctx.obs_data()
            .unwrap()
            .sv_epoch()
            .filter_map(|(epoch, svs)| if epoch == t { Some(svs) } else { None })
            .reduce(|svs, _| svs)
    }
}
