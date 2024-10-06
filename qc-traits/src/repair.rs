#[derive(Debug, Copy, Clone)]
pub enum Repairment {
    /// Repairs all zero values by simply removing them.
    DiscardZero,
    /// Apply static offset so data starts with 0/null value (y(t=0)=0).
    NullOrigin,
    /// Applies static offset to dataset
    Offset(f64),
    /// Applies static a*t+b law to dataset (a, b)
    ScalingOffset((f64, f64)),
}

pub trait Repair {
    fn repair(&self, r: Repairment) -> Self {
        let mut s = self.clone();
        s.repair_mut(r);
        s
    }
    fn repair_mut(&mut self, r: Repairment);
}
