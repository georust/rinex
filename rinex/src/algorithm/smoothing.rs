/// Smoothing filters
pub trait Smoothing {
	/// Hatch smoothing filter, only applies to Pseudo Range observables.
	/// Returns smoothing pseudo range observations.
	fn hatch_filter(&self) -> Self;
	/// Applies Hatch filter in place
	fn hatch_filter_mut(&mut self);
}
