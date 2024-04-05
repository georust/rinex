//! Observation RINEX module
use super::{epoch, prelude::*, version::Version};
use std::collections::HashMap;

pub mod record;

pub mod flag;
pub use flag::EpochFlag;

mod snr;
pub use snr::SNR;

#[cfg(docrs)]
use crate::Bibliography;

pub use record::{LliFlags, ObservationData, Record};

macro_rules! fmt_month {
    ($m: expr) => {
        match $m {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            _ => "Dec",
        }
    };
}

#[cfg(feature = "serde")]
use serde::Serialize;

/// Describes `Compact RINEX` specific information
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Crinex {
    /// Compression program version
    pub version: Version,
    /// Compression program name
    pub prog: String,
    /// Date of compression
    pub date: Epoch,
}

impl Crinex {
    /// Sets compression algorithm revision
    pub fn with_version(&self, version: Version) -> Self {
        let mut s = self.clone();
        s.version = version;
        s
    }
    /// Sets compression program name
    pub fn with_prog(&self, prog: &str) -> Self {
        let mut s = self.clone();
        s.prog = prog.to_string();
        s
    }
    /// Sets compression date
    pub fn with_date(&self, e: Epoch) -> Self {
        let mut s = self.clone();
        s.date = e;
        s
    }
}

impl Default for Crinex {
    fn default() -> Self {
        Self {
            version: Version { major: 3, minor: 0 },
            prog: format!("rust-rinex-{}", env!("CARGO_PKG_VERSION")),
            date: epoch::now(),
        }
    }
}

impl std::fmt::Display for Crinex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let version = self.version.to_string();
        write!(f, "{:<width$}", version, width = 20)?;
        write!(f, "{:<width$}", "COMPACT RINEX FORMAT", width = 20)?;
        writeln!(
            f,
            "{value:<width$} CRINEX VERS   / TYPE",
            value = "",
            width = 19
        )?;
        write!(f, "{:<width$}", self.prog, width = 20)?;
        write!(f, "{:20}", "")?;
        let (y, m, d, hh, mm, _, _) = self.date.to_gregorian_utc();
        let m = fmt_month!(m);
        let date = format!("{:02}-{}-{} {:02}:{:02}", d, m, y - 2000, hh, mm);
        write!(f, "{:<width$}", date, width = 20)?;
        f.write_str("CRINEX PROG / DATE")
    }
}

/// Observation Record specific header fields
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Optional CRINEX information
    pub crinex: Option<Crinex>,
    /// Time of FIRST OBS
    pub time_of_first_obs: Option<Epoch>,
    /// Time of LAST OBS
    pub time_of_last_obs: Option<Epoch>,
    /// Observables per constellation basis
    pub codes: HashMap<Constellation, Vec<Observable>>,
    /// True if local clock drift is compensated for
    pub clock_offset_applied: bool,
    /// Possible observation scaling, used in high precision
    /// OBS RINEX (down to nano radians precision).
    pub scaling: HashMap<(Constellation, Observable), u16>,
}

impl HeaderFields {
    /// Add TIME OF FIRST OBS
    pub(crate) fn with_time_of_first_obs(&self, epoch: Epoch) -> Self {
        let mut s = self.clone();
        s.time_of_first_obs = Some(epoch);
        s
    }
    /// Add TIME OF LAST OBS
    pub(crate) fn with_time_of_last_obs(&self, epoch: Epoch) -> Self {
        let mut s = self.clone();
        s.time_of_last_obs = Some(epoch);
        s
    }
    /// Insert a data scaling
    pub(crate) fn with_scaling(&mut self, c: Constellation, observable: Observable, scaling: u16) {
        self.scaling.insert((c, observable.clone()), scaling);
    }
    /// Returns given scaling to apply for given GNSS system
    /// and given observation. Returns 1.0 by default, so it always applies
    pub(crate) fn scaling(&self, c: Constellation, observable: Observable) -> Option<&u16> {
        self.scaling.get(&(c, observable))
    }
}

#[cfg(feature = "obs")]
use std::collections::BTreeMap;

#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
#[derive(Debug, Copy, Clone)]
pub enum Combination {
    GeometryFree,
    IonosphereFree,
    WideLane,
    NarrowLane,
    MelbourneWubbena,
}

/// GNSS signal combination trait.    
/// This only applies to OBS RINEX records.  
/// Refer to [Bibliography::ESAGnssCombination] and [Bibliography::ESABookVol1]
/// for more information.
#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
pub trait Combine {
    fn combine(
        &self,
        combination: Combination,
    ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>;
}

/// GNSS code bias estimation trait.
/// Refer to [Bibliography::ESAGnssCombination] and [Bibliography::ESABookVol1].
#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
pub trait Dcb {
    /// Returns Differential Code Bias estimates, sorted per (unique)
    /// signals combinations and for each individual SV.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observation::*; // .dcb()
    ///
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///    .unwrap();
    /// let dcb = rinex.dcb();
    /// ```
    fn dcb(&self) -> HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>;
}

#[cfg(test)]
mod crinex {
    use super::*;
    #[test]
    fn test_fmt_month() {
        assert_eq!(fmt_month!(1), "Jan");
        assert_eq!(fmt_month!(2), "Feb");
        assert_eq!(fmt_month!(3), "Mar");
        assert_eq!(fmt_month!(10), "Oct");
        assert_eq!(fmt_month!(11), "Nov");
        assert_eq!(fmt_month!(12), "Dec");
    }
    #[test]
    fn test_display() {
        let crinex = Crinex::default();
        let now = Epoch::now().unwrap();
        let (_y, _m, _d, _hh, _mm, _, _) = now.to_gregorian_utc();
        let content = crinex.to_string();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2); // main title should span 2 lines

        // test first line
        let expected =
            "3.0                 COMPACT RINEX FORMAT                    CRINEX VERS   / TYPE";
        assert_eq!(expected, lines[0]);

        // test second line width : must follow RINEX standards
        //assert_eq!(lines[1].len(), 80);
    }
}
