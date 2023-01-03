use crate::prelude::*;
use std::collections::HashMap;
use crate::observation::Record;

pub trait Processing {
	/// Evaluates minimal observation for all signals and Sv
	fn min(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
	/// Evalutates minimal observation for all signals accross all Sv
	fn min_observable(&self) -> HashMap<Observable, f64>;
    
	/// Evaluates maximal observation, for all signals and Sv	
	fn max(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
	/// Evaluates maximal observation for all signals
	fn max_observable(&self) -> HashMap<Observable, f64>;
	
    /// Evaluates average value for all signals and Sv
	fn mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
	/// Evalutes average value for all signals (averages accross Sv)
	fn mean_observable(&self) -> HashMap<Observable, f64>;

    /// Evaluates observation skewness per Sv per signal
    fn skewness(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    fn skewness_observable(&self) -> HashMap<Observable, f64>;

	/// Evaluates standard deviation for all signals and Sv
	fn stddev(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
	/// Evaluates standard deviation for all signals
	fn stddev_observable(&self) -> HashMap<Observable, f64>;
	
    /// Evaluates standard variance for all signals and Sv
	fn stdvar(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
	/// Evaluates standard deviation for all signals 
	fn stdvar_observable(&self) -> HashMap<Observable, f64>;

    /// Evaluates nth order central moment for all Signals for all Sv
	fn central_moment(&self, order: u16) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>);
    /// Evaluates nth order central moment for all Signals, accross Sv
	fn central_moment_observable(&self, order: u16) -> HashMap<Observable, f64>;
    
    fn derivative(&self) -> Record;
    // /// computes nth order derivative of this subset
    // fn derivative_nth(&self, order: u8) -> A;
}   
