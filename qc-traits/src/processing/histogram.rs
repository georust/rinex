pub type Histogram<T> = Vec<HistogramEntry<T>>;

pub struct HistogramEntry<T> {
    // Number of entries (count) for this Item value
    pub population: usize,
    // Actual value
    pub value: T,
}
