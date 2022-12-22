pub trait Statistical<A> {
    /// Determines min value of this subset
    fn min(&self) -> A;
    /// Determines max value of this subset
    fn max(&self) -> A;
    /// Determines median value of this subset
    fn median(&self) -> A;
    /// Computes statitical mean of this dataset
    fn mean(&self) -> A;
    /// Computes (absolute) error between self and given model
    fn error(&self, model: A) -> A;
    /// Computes the statistical mean of the absolute error
    /// between self and given model
    fn mean_err(&self, model: A) -> A;
    /// Computes std dev of this dataset
    fn stddev(&self) -> A;
    /// Computes nth order central moment, nth = 2 is [Self::stddev]
    fn central_moment(&self, order: u16) -> A;
    /// Computes weighted sum of this dataset
    fn weighted_sum(&self, w: A) -> A;
    /// Computes weighted mean of this dataset
    fn weighted_mean(&self, w: A) -> A;
    /// Skewness / Pearson's moment coefficient
    fn skewness(&self) -> A;
    /// L2 distance (squared or not) between self and given subset
    fn l2_distance(&self, squared: bool, rhs: A) -> A;
    // /// Computes statistical means for this dataset over specified dt duration 
    // fn mean_series(&self, dt: Duration) -> DataTimeSerie<f64>;
}
