//! RINEX Clock files parser & analysis
use hifitime::TimeScale;
pub mod record;
pub use record::{ClockData, ClockDataType, Error, Record, System};

/// Clocks `RINEX` specific header fields
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Types of observation in this file
    pub codes: Vec<ClockDataType>,
    /// Clock Data analysis production center
    pub agency: Option<ClockAnalysisAgency>,
    /// Reference station
    pub station: Option<Station>,
    /// Reference clock descriptor
    pub clock_ref: Option<String>,
    /// Timescale, can be a GNSS timescale, or UTC, TAI..
    /// also omitted for SBAS and COMPASS files
    pub timescale: Option<TimeScale>,
}

impl HeaderFields {
    /// Sets Reference clock description
    pub fn with_ref_clock(&self, clock: &str) -> Self {
        let mut s = self.clone();
        s.clock_ref = Some(clock.to_string());
        s
    }
    /// Set reference station
    pub fn with_ref_station(&self, station: Station) -> Self {
        let mut s = self.clone();
        s.station = Some(station);
        s
    }
    /// Set timescale
    pub fn with_timescale(&self, timescale: TimeScale) -> Self {
        let mut s = self.clone();
        s.timescale = Some(timescale);
        s
    }
    /// Set production agency
    pub fn with_agency(&self, agency: ClockAnalysisAgency) -> Self {
        let mut s = self.clone();
        s.agency = Some(agency);
        s
    }
}

/// Describes a clock station
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Station {
    /// Station name
    pub name: String,
    /// Station official ID#
    pub id: String,
}

/// Describes a clock analysis agency
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ClockAnalysisAgency {
    /// IGS AC 3 letter code
    pub code: String,
    /// agency name
    pub name: String,
}
