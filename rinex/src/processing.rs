use super::Rinex;

/// Context 1D is the combination
/// of Observation RINEX and Navigation frames,
/// ideally sampled at the same time.
pub struct Context1D {
    pub observations: Rinex,
    pub ephemeris: Rinex,
}

impl Context1d {
    pub fn new (observations: Rinex, ephemeris: Rinex) -> Result<Self, Error> {
        Self {
            observations,
            ephemeris,
        }
    }
    pub fn with_observations(&self, observations: Rinex) -> Result<Self, Error> {
        Self {
            observations,
            ephemeris: self.ephemeris,
        }
    }
    pub fn with_ephemeris(&self, ephemeris: Rinex) -> Result<Self, Error> {
        Self {
            observations: self.observations,
            ephemeris,
        }
    }
}

/// DiffContext is a structure to help perform
/// RINEX differentiation operations
pub struct DiffContext {
    pub context_a: Context1D,
    pub context_b: Context1D,
}

impl DiffContext {}
