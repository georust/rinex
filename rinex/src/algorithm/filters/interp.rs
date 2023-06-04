use super::TargetItem;
use crate::TimeSeries;

#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub enum InterpMethod {
    Linear,
}

#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct InterpFilter {
    pub series: TimeSeries,
    pub method: InterpMethod,
    pub target: Option<TargetItem>,
}

/*
 * Interpolates yp at xp
 */
pub(crate) fn lerp(x0y0: (f64, f64), x1y1: (f64, f64), xp: f64) -> f64 {
    let (x0, y0) = x0y0;
    let (x1, y1) = x1y1;
    y0 * (x1 - xp) + y1 * (xp - x0) / (x1 - x0)
}

/*
impl InterpFilter {
    fn linear_interp(y: Vec<f64>) -> Vec<f64> ;
    fn approx_linear_interp(y: Vec<f64>) -> Vec<f64> ;
    /// Applies self to given vector
    pub fn interp(y: Vec<f64>) -> Vec<f64> {

    }
    /// Applies self in place to input vector
    pub fn interp_mut(&mut y: Vec<f64>) {

    }
}*/

pub trait Interpolate {
    fn interpolate(&self, series: TimeSeries, target: Option<TargetItem>) -> Self;
    fn interpolate_mut(&mut self, series: TimeSeries, target: Option<TargetItem>);
}
