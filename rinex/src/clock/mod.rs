//! RINEX Clock files parser & analysis
pub mod record;

pub use record::{ClockKey, ClockProfile, ClockProfileType, ClockType, Error, Record};

use crate::version::Version;
use hifitime::TimeScale;
use std::str::FromStr;

use crate::domes::DOMES;

/// Clocks `RINEX` specific header fields
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Site name
    pub site: Option<String>,
    /// Site DOMES ID#
    pub domes: Option<DOMES>,
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
    /// Clock site DOMES ID#
    pub domes: Option<DOMES>,
    /// Possible clock constraint [s]
    pub constraint: Option<f64>,
}

impl WorkClock {
    pub(crate) fn parse(version: Version, content: &str) -> Self {
        const LIMIT: Version = Version { major: 3, minor: 4 };
        if version < LIMIT {
            let (name, rem) = content.split_at(4);
            let (domes, rem) = rem.split_at(36);
            let constraint = rem.split_at(20).0;
            Self {
                name: name.trim().to_string(),
                domes: if let Ok(domes) = DOMES::from_str(domes.trim()) {
                    Some(domes)
                } else {
                    None
                },
                constraint: if let Ok(value) = constraint.trim().parse::<f64>() {
                    Some(value)
                } else {
                    None
                },
            }
        } else {
            let (name, rem) = content.split_at(10);
            let (domes, rem) = rem.split_at(10);
            let constraint = rem.split_at(40).0;
            Self {
                name: name.trim().to_string(),
                domes: if let Ok(domes) = DOMES::from_str(domes.trim()) {
                    Some(domes)
                } else {
                    None
                },
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
    pub(crate) fn work_clock(&self, clk: WorkClock) -> Self {
        let mut s = self.clone();
        s.work_clock.push(clk);
        s
    }
    pub(crate) fn timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.timescale = Some(ts);
        s
    }
    pub(crate) fn site(&self, site: &str) -> Self {
        let mut s = self.clone();
        s.site = Some(site.to_string());
        s
    }
    pub(crate) fn domes(&self, domes: DOMES) -> Self {
        let mut s = self.clone();
        s.domes = Some(domes);
        s
    }
    pub(crate) fn igs(&self, igs: &str) -> Self {
        let mut s = self.clone();
        s.igs = Some(igs.to_string());
        s
    }
    pub(crate) fn full_name(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.full_name = Some(name.to_string());
        s
    }
    pub(crate) fn refclock(&self, clk: &str) -> Self {
        let mut s = self.clone();
        s.ref_clock = Some(clk.to_string());
        s
    }
}
