use super::{
    Constellation,
    version::Version,
};

pub mod record;
pub use record::{
    Record, LliFlags, Ssi,
    is_new_epoch,
    fmt_epoch,
    parse_epoch,
};

use std::collections::HashMap;

macro_rules! fmtmonth {
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
            10=> "Oct",
            11=> "Nov",
            _ => "Dec",
        }
    }
}

#[cfg(feature = "serde")]
use serde::Serialize;

/// Describes `Compact RINEX` specific information
#[derive(Clone, Debug)]
#[derive(PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Crinex {
    /// Compression program version
    pub version: Version,
    /// Compression program name
    pub prog: String,
    /// Date of compression
    pub date: hifitime::Epoch,
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
    pub fn with_date(&self, e: hifitime::Epoch) -> Self {
        let mut s = self.clone();
        s.date = e;
        s
    }
}

impl Default for Crinex {
    fn default() -> Self {
        Self {
            version: Version {
                major: 3,
                minor: 0,
            },
            prog: "rust-crinex".to_string(),
            date: hifitime::Epoch::now()
                .expect("failed to retrieve system time"),
        }
    }
}

impl std::fmt::Display for Crinex {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let version = self.version.to_string();
        write!(f, "{:<width$}", version, width=20)?;
        write!(f, "{:<width$}", "COMPACT RINEX FORMAT", width=20)?;
        write!(f, "{value:<width$} CRINEX VERS   / TYPE\n", value="", width=19)?;
        write!(f, "{:<width$}", self.prog, width=20)?;
        write!(f, "{:20}", "")?;
        let (mut y, m, d, hh, mm, _, _) = self.date.to_gregorian_utc();
        let m = fmtmonth!(m);
        // we want a 2 digit year
        if y > 2000 {
            y -= 2000;
        }
        if y > 1900 {
            y -= 1900;
        }
        let date = format!("{:02}-{}-{} {:02}:{:02}", d, m, y, hh, mm);
        write!(f, "{:<width$}", date, width=20)?;
        f.write_str("CRINEX PROG / DATE\n")
    }
}

/// Describes known marker types
/// Observation Record specific header fields
#[derive(Debug, Clone, Default)]
#[derive(PartialEq, Eq)]
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
        if let Some(scalings) = self.scalings.get_mut(&c) {
            scalings.insert(observation.to_string(), scaling);
        } else {
            let mut map: HashMap<String, f64> = HashMap::new();
            map.insert(observation.to_string(), scaling);
            self.scalings.insert(c, map);
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
            for comp in self.dcb_compensations {
                if comp == c {
                    return true ;
                }
            }
            false
        
        } else {
            for (cst, _) in self.codes { // all encountered constellations
                let mut found = false;
                for ccst in self.dcb_compensations { // all compensated constellations
                    if ccst == cst {
                        found = true;
                        break ;
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
