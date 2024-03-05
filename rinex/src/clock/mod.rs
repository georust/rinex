//! RINEX Clock files parser & analysis
pub mod record;

pub use record::{ClockKey, ClockProfile, ClockProfileType, ClockType, Error, Record};

use crate::version::Version;
use hifitime::TimeScale;

/// Clocks `RINEX` specific header fields
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Site name
    pub site: Option<String>,
    /// Unique Site-ID (DOMES number)
    pub site_id: Option<String>,
    /// IGS code
    pub igs: Option<String>,
    /// Full name
    pub full_name: Option<String>,
    /// Station reference clock
    pub ref_clock: Option<String>,
    /// Timescale is either a GNSS timescale or UTC / TAI.
    /// Timescale is omitted in SBAS or COMPASS files.
    pub timescale: Option<TimeScale>,
    /// Reference clocks used in measurement / analysis process
    pub work_clock: Vec<WorkClock>,
    /// Types of clock profiles encountered in this file
    pub codes: Vec<ClockProfileType>,
}

/// Clock used in the analysis and evaluation of this file
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkClock {
    /// Name of this local clock
    pub name: String,
    /// Unique identifier (DOMES number)
    pub id: String,
    /// Possible clock constraint [s]
    pub constraint: Option<f64>,
}

impl WorkClock {
    pub(crate) fn parse(version: Version, content: &str) -> Self {
        const LIMIT: Version = Version { major: 3, minor: 4 };
        if version < LIMIT {
            let (name, rem) = content.split_at(4);
            let (id, rem) = rem.split_at(36);
            let constraint = rem.split_at(20).0;
            Self {
                name: name.trim().to_string(),
                id: id.trim().to_string(),
                constraint: if let Ok(value) = constraint.trim().parse::<f64>() {
                    Some(value)
                } else {
                    None
                },
            }
        } else {
            let (name, rem) = content.split_at(10);
            let (id, rem) = rem.split_at(10);
            let constraint = rem.split_at(40).0;
            Self {
                name: name.trim().to_string(),
                id: id.trim().to_string(),
                constraint: if let Ok(value) = constraint.trim().parse::<f64>() {
                    Some(value)
                } else {
                    None
                },
            }
        }
    }
}

impl HeaderFields {
    pub fn work_clock(&self, clk: WorkClock) -> Self {
        let mut s = self.clone();
        s.work_clock.push(clk);
        s
    }
    pub fn timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.timescale = Some(ts);
        s
    }
    pub fn site(&self, site: &str) -> Self {
        let mut s = self.clone();
        s.site = Some(site.to_string());
        s
    }
    pub fn site_id(&self, siteid: &str) -> Self {
        let mut s = self.clone();
        s.site_id = Some(siteid.to_string());
        s
    }
    pub fn igs(&self, igs: &str) -> Self {
        let mut s = self.clone();
        s.igs = Some(igs.to_string());
        s
    }
    pub fn full_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.full_name = Some(name.to_string());
        s
    }
    pub fn refclock(&self, clk: &str) -> Self {
        let mut s = self.clone();
        s.ref_clock = Some(clk.to_string());
        s
    }
}
