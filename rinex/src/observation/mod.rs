//! Observation RINEX module
use super::{epoch, prelude::*, version::Version};
use std::collections::HashMap;

pub mod record;
mod snr;

#[cfg(docrs)]
use crate::Bibliography;

pub use record::{LliFlags, ObservationData, Record};
pub use snr::Snr;

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

/// DCB Compensation description
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DcbCompensation {
    /// Program used for DCBs evaluation and compensation
    pub program: String,
    /// Constellation to which this compensation applies to
    pub constellation: Constellation,
    /// URL: source of corrections
    pub url: String,
}

/// Describes known marker types
/// Observation Record specific header fields
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Optional CRINEX information
    pub crinex: Option<Crinex>,
    /// Observables per constellation basis
    pub codes: HashMap<Constellation, Vec<Observable>>,
    /// True if local clock drift is compensated for
    pub clock_offset_applied: bool,
    /// DCBs compensation per constellation basis
    pub dcb_compensations: Vec<DcbCompensation>,
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
    pub(crate) fn scaling(&self, c: &Constellation, observable: &Observable) -> Option<&f64> {
        let scalings = self.scalings.get(c)?;
        scalings.get(observable)
    }

    /// Emphasize that DCB is compensated for
    pub fn append_dcb_compensation(&mut self, dcb: DcbCompensation) {
        self.dcb_compensations.push(dcb);
    }
}

#[cfg(feature = "obs")]
use std::collections::BTreeMap;

/// GNSS signal recombination trait.    
/// Import this to recombine OBS RINEX with usual recombination methods.   
/// This only applies to OBS RINEX records.  
/// Refer to [Bibliography::ESAGnssCombination] and [Bibliography::ESABookVol1]
/// for more information.
#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
pub trait Combine {
    /// Perform Geometry Free signal recombination on all phase
    /// and pseudo range observations, for each individual Sv
    /// and individual Epoch.   
    /// Geometry Free (Gf) recombination cancels out geometric
    /// biases and leaves frequency dependent terms out,
    /// like Ionospheric induced time delay.  
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observation::*;
    ///
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///		.unwrap();
    ///
    /// let gf = rinex.geo_free();
    /// for ((ref_observable, rhs_observable), data) in gf {
    ///     // for each recombination that we were able to form,
    ///     // a "reference" observable was chosen,
    ///     // and RHS observable is compared to it.
    ///     // For example "L2C-L1C" : L1C is the reference observable
    ///     for (sv, epochs) in data {
    ///         // applied to all possible Sv
    ///         for ((epoch, _flag), value) in epochs {
    ///             // value: actual recombination result
    ///         }
    ///     }
    /// }
    /// ```
    fn geo_free(
        &self,
    ) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>;

    /// Perform Wide Lane recombination.   
    /// See [Self::geo_free] for API example.
    fn wide_lane(
        &self,
    ) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>;

    /// Perform Narrow Lane recombination.   
    /// See [Self::geo_free] for API example.
    fn narrow_lane(
        &self,
    ) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>;

    /// Perform Melbourne-Wübbena recombination.   
    /// See [`Self::geo_free`] for API example.
    fn melbourne_wubbena(
        &self,
    ) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>;
}

/// GNSS code bias estimation trait.
/// Refer to [Bibliography::ESAGnssCombination] and [Bibliography::ESABookVol1].
#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
pub trait Dcb {
    /// Returns Differential Code Bias estimates, sorted per (unique)
    /// signals combinations and for each individual Sv.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observation::*; // .dcb()
    ///
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///		.unwrap();
    /// let dcb = rinex.dcb();
    /// ```
    fn dcb(&self) -> HashMap<String, BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>;
}

/// Multipath biases estimation.
/// Refer to [Bibliography::ESABookVol1] and [Bibliography::MpTaoglas].
#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
pub trait Mp {
    /// Returns Multipath bias estimates,
    /// sorted per (unique) signal combinations and for each individual Sv.
    fn mp(&self) -> HashMap<String, BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>;
}

/// Ionospheric Delay estimation trait.
#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
pub trait IonoDelay {
    /// The Iono delay estimator is the derivative of the [Combine::geo_free]
    /// recombination. One can then use a peak detector for example,
    /// to determine signal perturbations, due to ionospheric activity.
    /// To improve behavior and avoid discontinuities on data gaps,
    /// we perform the derivative only if the previous point was sampled at worst
    /// `max_dt` prior current point.  
    /// This is intended to be used on raw Phase data only,
    /// but can be evaluated on PR too (if such data is passed).  
    /// In that scenario, ideally the user used a smoothing algorithm,
    /// prior to invoking this method: see the preprocessing toolkit.
    fn iono_delay(
        &self,
        max_dt: Duration,
    ) -> HashMap<Observable, HashMap<Sv, BTreeMap<Epoch, f64>>>;
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
