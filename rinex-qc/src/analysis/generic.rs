pub trait QcAnalysis<M, R> {

    /// Returns true when this analysis should not be rendered
    /// (ie., contribute to the final report)
    fn is_null(&self) -> bool;

    /// Latch a new measurement that will contribute to this analysis
    fn new_measurement(&mut self, data: M);

    /// Convert this analysis to visual, ready to render
    fn render(&self) -> R;
}