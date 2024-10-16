use std::ops::Rem;

use crate::prelude::ParsingError;

/// Linear space as used in IONEX or Antenna grid definitions.
/// Linear space starting from `start` ranging to `end` (included).
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Linspace {
    /// start coordinates or value
    pub start: f64,
    /// end coordinates or value
    pub end: f64,
    /// spacing (increment value)
    pub spacing: f64,
}

impl Linspace {
    /// Builds a new Linear space
    pub fn new(start: f64, end: f64, spacing: f64) -> Result<Self, ParsingError> {
        let r = end.rem(start);
        /*
         * End / Start must be multiple of one another
         */
        if r == 0.0 {
            if end.rem(spacing) == 0.0 {
                Ok(Self {
                    start,
                    end,
                    spacing,
                })
            } else {
                Err(ParsingError::BadIonexGridSpecs)
            }
        } else {
            Err(ParsingError::BadIonexGridSpecs)
        }
    }
    // Returns grid length, in terms of data points
    pub fn length(&self) -> usize {
        (self.end / self.spacing).floor() as usize
    }
    /// Returns true if self is a single point space
    pub fn is_single_point(&self) -> bool {
        (self.end == self.start) && self.spacing == 0.0
    }
}

impl From<(f64, f64, f64)> for Linspace {
    fn from(tuple: (f64, f64, f64)) -> Self {
        Self {
            start: tuple.0,
            end: tuple.1,
            spacing: tuple.2,
        }
    }
}
