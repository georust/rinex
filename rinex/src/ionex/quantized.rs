//! IONEX Grid Quantization

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quantized<T> {
    value: T,
    spacing: T,
    q_factor: T,
}

impl<T> Quantized<T> {
    pub fn new(value: f64, spacing: f64) -> Self {
        Self {
            value,
            spacing,
            q_factor,
        }
    }

    pub fn value(&self) -> f64 {
        self.value as f64 * self.q_factor as f64 / self.spacing as f64
    }
}