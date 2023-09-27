use nalgebra::base::{DVector, Matrix4xX, Vector4};
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
    /// Position Dilution of Precision
    pub tdop: f64,
    /// Time (only) Dilution of Precision
    pub pdop: f64,
}

impl SolverEstimate {
    /*
     * Builds a new SolverEstimate from `g` Nav Matrix,
     * and `y` Nav Vector
     */
    pub fn new(g: Matrix4xX<f64>, y: DVector<f64>) -> Option<Self> {
        let g_prime = g.transpose();
        let q = (g.clone() * g_prime.clone()).try_inverse()?;
        let x = q * g_prime.clone() * y;
        let pdop = (q[(1, 1)] + q[(2, 2)] + q[(3, 3)]).sqrt();
        let tdop = q[(4, 4)].sqrt();
        Some(Self {
            dx: x[0],
            dy: x[1],
            dz: x[2],
            dt: x[3],
            tdop,
            pdop,
        })
    }
}
