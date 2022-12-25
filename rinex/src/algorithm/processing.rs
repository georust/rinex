pub trait Processing<A> {
	fn mean(&self) -> A;
	fn stddev(&self) -> A;
	fn skewness(&self) -> A;
	fn central_moment(&self, order: u16) -> A;
/*
	/// averages this subset with desired method
    fn average(&self) -> A;
    /// computes 1st order derivative of this subset
    fn derivative(&self) -> A;
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
