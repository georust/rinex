//! Antex - special RINEX type specific structures
use crate::channel;

#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Pattern {
    /// Non azimuth dependent pattern
    NonAzimuthDependent(Vec<f64>),
    /// Azimuth dependent pattern
    AzimuthDependent((f64, Vec<f64>)),
}

impl Default for Pattern {
    fn default() -> Self {
        Self::NonAzimuthDependent(Vec::new())
    }
}

impl Pattern {
    /// Returns true if this phase pattern is azimuth dependent
    pub fn is_azimuth_dependent (&self) -> bool {
        match self {
            Self::AzimuthDependent(_) => true,
            _ => false,
        }
    }
    /// Unwraps pattern values, whether it is
    /// Azimuth dependent or not
    pub fn pattern (&self) -> Vec<f64> {
        match self {
            Self::AzimuthDependent((_, values)) => values.clone(),
            Self::NonAzimuthDependent(values) => values.clone(),
        }
    }
    /// Unwraps pattern and associated azimuth angle,
    /// in case of azimuth dependent phase pattern
    pub fn azimuth_pattern (&self) -> Option<(f64, Vec<f64>)> {
        match self {
            Self::AzimuthDependent((angle, values)) => Some((*angle, values.clone())),
            _ => None
        }
    }
}

/// Describes "frequency" data attached to a specific Antenna
/// in the ATX record
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Frequency {
    /// Channel, example: L1, L2 for GPS, E1, E5 for GAL...
    pub channel: channel::Channel,
    /// Northern component of the mean antenna phase center
    /// relative to the antenna reference point (ARP),
    /// in [mm]
    pub north: f64,
    /// Eastern component of the mean antenna phase center
    /// relative to the antenna reference point (ARP),
    /// in [mm]
    pub east: f64,
    /// Z component of the mean antenna phase center relative
    /// to the antenna reference point (ARP),
    /// in [mm]
    pub up: f64,
    /// Phase pattern, values in [mm] from antenna.zen1 to antenna.zen2
    /// with increment antenna.dzen, can either be Azimuth or NonAzimmuth
    /// dependent
    pub patterns: Vec<Pattern>, 
}

impl Default for Frequency {
    fn default() -> Self {
        Self {
            channel: channel::Channel::default(),
            north: 0.0_f64,
            east: 0.0_f64,
            up: 0.0_f64,
            patterns: Vec::new(),
        }
    }
}

impl Frequency {
    pub fn with_channel (&self, channel: channel::Channel) -> Self {
        let mut f = self.clone();
        f.channel = channel.clone();
        f
    }
    pub fn with_northern_eccentricity (&self, north: f64) -> Self {
        let mut f = self.clone();
        f.north = north;
        f
    }
    pub fn with_eastern_eccentricity (&self, east: f64) -> Self {
        let mut f = self.clone();
        f.east = east;
        f
    }
    pub fn with_upper_eccentricity (&self, up: f64) -> Self {
        let mut f = self.clone();
        f.up = up;
        f
    }
    pub fn add_pattern (&self, p: Pattern) -> Self {
        let mut f = self.clone();
        f.patterns.push(p.clone());
        f
    }
}
