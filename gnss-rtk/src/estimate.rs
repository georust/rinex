use nyx_space::cosmic::SPEED_OF_LIGHT;
// use nalgebra::linalg::svd::SVD;
use nalgebra::base::{DVector, MatrixXx4};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/*
 * Solver solution estimate
 * is always expressed as a correction of an 'a priori' position
*/
#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SolverEstimate {
    /// X coordinates correction
    pub dx: f64,
    /// Y coordinates correction
    pub dy: f64,
    /// Z coordinates correction
    pub dz: f64,
    /// Time correction
    pub dt: f64,
    /// Dilution of Position Precision, horizontal component
    pub hdop: f64,
    /// Dilution of Position Precision, vertical component
    pub vdop: f64,
    /// Time Dilution of Precision
    pub tdop: f64,
}

impl SolverEstimate {
    /*
     * Builds a new SolverEstimate from `g` Nav Matrix,
     * and `y` Nav Vector
     */
    pub fn new(g: MatrixXx4<f64>, y: DVector<f64>) -> Option<Self> {
        //let svd = g.clone().svd(true, true);
        //let u = svd.u?;
        //let v = svd.v_t?;
        //let s = svd.singular_values;
        //let s_inv = s.pseudo_inverse(1.0E-8).unwrap();
        //let x = v * u.transpose() * y * s_inv;

        let g_prime = g.clone().transpose();
        let q = (g_prime.clone() * g.clone()).try_inverse()?;
        let x = q * g_prime.clone();
        let x = x * y;

        let hdop = (q[(0, 0)] + q[(1, 1)]).sqrt();
        let vdop = q[(2, 2)].sqrt();
        let tdop = q[(3, 3)].sqrt();

        Some(Self {
            dx: x[0],
            dy: x[1],
            dz: x[2],
            dt: x[3] / SPEED_OF_LIGHT,
            hdop,
            vdop,
            tdop,
        })
    }
}
