use super::{epoch, prelude::*, version::Version};
use std::collections::HashMap;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

pub mod record;
mod snr;

pub use record::{LliFlags, ObservationData, Record};
pub use snr::Snr;

#[cfg(feature = "pyo3")]
pub use record::PyRecord;

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
#[cfg_attr(feature = "pyo3", pyclass)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Crinex {
    /// Compression program version
    pub version: Version,
    /// Compression program name
    pub prog: String,
    /// Date of compression
    pub date: Epoch,
}

#[cfg_attr(feature = "pyo3", pymethods)]
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
    #[cfg(feature = "pyo3")]
    #[new]
    fn new_py() -> Self {
        Self::default()
    }
    #[cfg(feature = "pyo3")]
    #[getter]
    fn get_version(&self) -> Version {
        self.version
    }
    #[cfg(feature = "pyo3")]
    #[getter]
    fn get_prog(&self) -> &str {
        &self.prog
    }
    #[cfg(feature = "pyo3")]
    #[getter]
    fn get_date(&self) -> Epoch {
        self.date
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
        write!(
            f,
            "{value:<width$} CRINEX VERS   / TYPE\n",
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

/// Describes known marker types
/// Observation Record specific header fields
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Optional CRINEX information
    pub crinex: Option<Crinex>,
    /// Observables per constellation basis
    pub codes: HashMap<Constellation, Vec<Observable>>,
    /// True if local clock drift is compensated for
    pub clock_offset_applied: bool,
    /// DCBs compensation per constellation basis
    pub dcb_compensations: Vec<Constellation>,
    /// Optionnal data scalings
    pub scalings: HashMap<Constellation, HashMap<Observable, f64>>,
}

impl HeaderFields {
    /// Add an optionnal data scaling
    pub fn with_scaling(&self, c: Constellation, observable: Observable, scaling: f64) -> Self {
        let mut s = self.clone();
        if let Some(scalings) = s.scalings.get_mut(&c) {
            scalings.insert(observable, scaling);
        } else {
            let mut map: HashMap<Observable, f64> = HashMap::new();
            map.insert(observable, scaling);
            s.scalings.insert(c, map);
        }
        s
    }
    /// Returns given scaling to apply for given GNSS system
    /// and given observation. Returns 1.0 by default, so it always applies
    pub fn scaling(&self, c: &Constellation, observable: Observable) -> f64 {
        if let Some(scalings) = self.scalings.get(c) {
            if let Some(scaling) = scalings.get(&observable) {
                return *scaling;
            }
        }
        1.0
    }

    /// Emphasize that DCB is compensated for
    pub fn with_dcb_compensation(&self, c: Constellation) -> Self {
        let mut s = self.clone();
        s.dcb_compensations.push(c);
        s
    }
    /// Returns true if DCB compensation was applied for given constellation.
    /// If constellation is None: we test against all encountered constellation
    pub fn dcb_compensation(&self, c: Option<Constellation>) -> bool {
        if let Some(c) = c {
            for comp in &self.dcb_compensations {
                if *comp == c {
                    return true;
                }
            }
            false
        } else {
            for (cst, _) in &self.codes {
                // all encountered constellations
                let mut found = false;
                for ccst in &self.dcb_compensations {
                    // all compensated constellations
                    if ccst == cst {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return false;
                }
            }
            true
        }
    }
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
        let (y, m, d, hh, mm, _, _) = now.to_gregorian_utc();
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
