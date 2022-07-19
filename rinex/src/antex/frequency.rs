//! Antex - special RINEX type specific structures
use crate::channel;

#[derive(Debug, Clone)]
pub enum Pattern {
    /// Non azimuth dependent pattern
    NonAzimuthDependent(Vec<f64>),
    /// Azimuth dependent pattern
    AzimuthDependent(Vec<f64>),
}

impl Default for Pattern {
    fn default() -> Self {
        Self::NonAzimuthDependent(Vec::new())
    }
}

/// Describes a "frequency" section of the ATX record
/// Describes a "frequency" section of the ATX record
#[derive(Debug, Clone)]
pub struct Frequency {
    /// Channel, example: L1, L2 for GPS, E1, E5 for GAL...
    pub channel: channel::Channel,
    /// TODO
    pub north: f64,
    /// TODO
    pub east: f64,
    /// TODO
    pub up: f64,
    /// Possibly azimuth dependent pattern
    pub pattern: Pattern, 
}

impl Default for Frequency {
    fn default() -> Self {
        Self {
            channel: channel::Channel::default(),
            north: 0.0_f64,
            east: 0.0_f64,
            up: 0.0_f64,
            pattern: Pattern::default(),
        }
    }
}

impl Frequency {
    /// Returns `Frequency` object with updated `Northern` component
    pub fn with_northern (&self, north: f64) -> Self {
        let mut f = self.clone();
        f.north = north;
        f
    }
    pub fn with_eastern (&self, east: f64) -> Self {
        let mut f = self.clone();
        f.east = east;
        f
    }
    pub fn with_upper (&self, up: f64) -> Self {
        let mut f = self.clone();
        f.up = up;
        f
    }
}
