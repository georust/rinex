//! Clock RINEX specific Header
use crate::{
    prelude::{
        DOMES,
        TimeScale,
    },
    clock::ClockProfile,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Clock [RINEX] specific header fields
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
    pub reference_clock: Option<String>,
    /// Timescale is either a GNSS timescale or UTC / TAI.
    /// Timescale is omitted in SBAS or COMPASS files.
    pub timescale: Option<TimeScale>,
    /// [Clock]s described / analyzed in this file
    pub clocks: Vec<Clock>,
    /// [ClockProfile]s found in this file
    pub codes: Vec<ClockProfileType>,
}

impl HeaderFields {
    /// Define [HeaderFields] with desired reference [Clock]
    pub fn with_clock(&self, clk: Clock) -> Self {
        let mut s = self.clone();
        s.clocks.push(clk);
        s
    }
    /// Define [HeaderFields] with desired [TimeScale]
    pub fn timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.timescale = Some(ts);
        s
    }
    /// Define [HeaderFields] with desired Site name
    pub fn site(&self, site: &str) -> Self {
        let mut s = self.clone();
        s.site = Some(site.to_string());
        s
    }
    /// Define [HeaderFields] with desired [DOMES] Site name
    pub fn domes(&self, domes: DOMES) -> Self {
        let mut s = self.clone();
        s.domes = Some(domes);
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
    pub fn reference_clock(&self, clk: &str) -> Self {
        let mut s = self.clone();
        s.reference_clock = Some(clk.to_string());
        s
    }
}
