pub enum Repairment {
    /// Repair all Zero Values by removing them
    Zero,
    /// Apply static offset to dataset (y'=x+b)
    Offset(f64),
    /// Apply static offset+scaling to dataset (y=ax+b)
    ScalingOffset((f64, f64)),
    /// Apply static offset so datasets starts with Zero/Null
    /// origin. That is, y(0)=0
    NullOrigin,
}

pub trait Repair {
    fn repair(&self, repairment: &Repairment) -> Self;
    fn repair_mut(&mut self, repairment: &Repairment);
}
