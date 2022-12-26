use std::collections::{BTreeMap, HashMap};
use crate::Sv;
use crate::Observable;
use crate::Epoch;

pub trait Processing {
	fn min(&self) -> HashMap<Sv, HashMap<Observable, f64>>;
	//fn min_sv(&self) -> HashMap<Sv, f64>;
	//fn min_observable(&self) -> HashMap<Observable, f64>;
	
	fn max(&self) -> HashMap<Sv, HashMap<Observable, f64>>;
	//fn max_sv(&self) -> HashMap<Sv, f64>;
	//fn max_observable(&self) -> HashMap<Observable, f64>;

	fn mean(&self) -> HashMap<Sv, HashMap<Observable, f64>>;
	//fn mean_sv(&self) -> HashMap<Sv, f64>;
	//fn mean_observable(&self) -> HashMap<Observable, f64>;

	fn derivative(&self) -> BTreeMap<Epoch, HashMap<Sv, HashMap<Observable, f64>>>;

	fn stddev(&self) -> HashMap<Sv, HashMap<Observable, f64>>;
	//fn stddev_sv(&self) -> HashMap<Sv, HashMap<Observable, f64>>;
	//fn stddev_observable(&self) -> HashMap<Sv, HashMap<Observable, f64>>;
	
	fn stdvar(&self) -> HashMap<Sv, HashMap<Observable, f64>>;

	//smoothing(&self) -> Self {}
	//smoothing_mut(&self);
	//interpolate(&self) -> Self;
	//interpolate_mut(&self);

/*
	fn skewness(&self) -> A;
	fn central_moment(&self, order: u16) -> A;
	fn derivative(&self) -> A;
	fn derivative_n(&self, n: u16) -> A;
	/// averages this subset with desired method
    fn average(&self) -> A;
    /// computes nth order derivative of this subset
    fn derivative_nth(&self, order: u8) -> A;
    /// applies smoothing to this subset
    fn smoothing(&self) -> A;
    fn smoothing_mut(&mut self);
	/// Interpolates self to macth the given time serie
	fn interpolate(&self, serie: hifitime::TimeSeries) -> A;
	fn interpolate_mut(&mut self, serie: hifitime::TimeSeries);
*/
}
