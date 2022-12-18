use super::{epoch, prelude::*, version::Version};

pub mod record;
pub use record::{LliFlags, ObservationData, Record, Ssi};

use std::collections::HashMap;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// Macro to determine whether this is a Phase observation or not
pub fn is_phase_observation(content: &str) -> bool {
    content.trim().starts_with("L")
}

/// Macro to determine whether this is a Doppler observation or not
pub fn is_doppler_observation(content: &str) -> bool {
    content.trim().starts_with("D")
}

/// Macro to determine whether this is a Pseudo Range observation or not
pub fn is_pseudorange_observation(content: &str) -> bool {
    content.trim().starts_with("C") || content.trim().starts_with("P")
}

/// Macro to determine whether this is an SSI observation or not
pub fn is_ssi_observation(content: &str) -> bool {
    content.trim().starts_with("S")
}

/// Macro to extract observation code
pub fn observation_code(content: &str) -> String {
    let c = content.trim();
    if c.len() > 2 {
        c[1..].to_string()
    } else {
        c[..std::cmp::min(c.len(), 2)].to_string()
    }
}

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
#[cfg_attr(feature = "pyo3", pyclass)]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Optional CRINEX information,
    /// only present on compressed OBS
    pub crinex: Option<Crinex>,
    /// Observation codes present in this file, by Constellation
    pub codes: HashMap<Constellation, Vec<String>>,
    /// True if epochs & data compensate for local clock drift
    pub clock_offset_applied: bool,
    /// List of constellation for which (Phase and PR)
    /// Differential Code Biases are compensated for
    pub dcb_compensations: Vec<Constellation>,
    /// Optionnal data scalings
    pub scalings: HashMap<Constellation, HashMap<String, f64>>,
}

impl HeaderFields {
    /// Add an optionnal data scaling
    pub fn with_scaling(&self, c: Constellation, observation: &str, scaling: f64) -> Self {
        let mut s = self.clone();
        if let Some(scalings) = s.scalings.get_mut(&c) {
            scalings.insert(observation.to_string(), scaling);
        } else {
            let mut map: HashMap<String, f64> = HashMap::new();
            map.insert(observation.to_string(), scaling);
            s.scalings.insert(c, map);
        }
        s
    }
    /// Returns given scaling to apply for given GNSS system
    /// and given observation. Returns 1.0 by default, so it always applies
    pub fn scaling(&self, c: &Constellation, observation: &String) -> f64 {
        if let Some(scalings) = self.scalings.get(c) {
            if let Some(scaling) = scalings.get(observation) {
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
        let expected = format!(
            "3.0                 COMPACT RINEX FORMAT                    CRINEX VERS   / TYPE
rust-rinex-{}                        {:02}-{}-{} {:02}:{:02}     CRINEX PROG / DATE",
            env!("CARGO_PKG_VERSION"),
            d,
            fmt_month!(m),
            y - 2000,
            hh,
            mm
        );
        assert_eq!(crinex.to_string(), expected);
    }
    #[test]
    fn test_observables() {
        assert_eq!(is_pseudorange_observation("C1P"), true);
        assert_eq!(is_pseudorange_observation("P1P"), true);
        assert_eq!(is_pseudorange_observation("L1P"), false);
        assert_eq!(is_phase_observation("L1P"), true);
        assert_eq!(is_phase_observation("D1P"), false);
        assert_eq!(is_doppler_observation("D1P"), true);
        assert_eq!(is_doppler_observation("L1P"), false);
        assert_eq!(is_ssi_observation("S1P"), true);
        assert_eq!(is_ssi_observation("L1P"), false);

        assert_eq!(observation_code("C1P"), "1P");
        assert_eq!(observation_code("C1"), "C1");
        assert_eq!(observation_code("L1C"), "1C");
        assert_eq!(observation_code("L1W"), "1W");
        assert_eq!(observation_code("L1"), "L1");
    }
}
