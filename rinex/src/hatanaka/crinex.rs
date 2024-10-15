//! CRINEX Header definitions
use crate::{
    epoch::now as epoch_now_utc,
    prelude::{Epoch, Version},
};

#[cfg(feature = "serde")]
use serde::Serialize;

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
/// CRINEX specifications
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CRINEX {
    /// Compression program version
    pub version: Version,
    /// Compression program name
    pub prog: String,
    /// Date of compression
    pub date: Epoch,
}

impl CRINEX {
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

impl Default for CRINEX {
    fn default() -> Self {
        Self {
            version: Version { major: 3, minor: 0 },
            prog: format!("rust-rinex-{}", env!("CARGO_PKG_VERSION")),
            date: epoch_now_utc(),
        }
    }
}

impl std::fmt::Display for CRINEX {
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

#[cfg(test)]
mod test {
    use super::CRINEX;
    use crate::prelude::Epoch;
    use std::str::FromStr;

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
    fn test_encode_decode() {
        let crinex = CRINEX {
            version: Version::new(3, 1),
            prog: "test".to_string(),
            date: Epoch::from_str("2010-10-09T00:30:45 UTC").unwrap(),
        };

        let formatted = crinex.to_string();
        let lines = formatted.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2, "formatted CRINEX should span 2 lines");

        assert_eq!(
            lines[0],
            "3.1                 COMPACT RINEX FORMAT                    CRINEX VERS   / TYPE"
        );

        assert_eq!(lines[1], "");

        let decoded = CRINEX::from_str(formatted);
        assert_eq!(decoded, crinex);

        let crinex = CRINEX {
            version: Version::new(2, 11),
            prog: "test".to_string(),
            date: Epoch::from_str("2015-20-10T09:08:07 UTC").unwrap(),
        };

        let formatted = crinex.to_string();
        let lines = formatted.lines().collect::<Vec<_>>();

        assert_eq!(lines.len(), 2, "formatted CRINEX should span 2 lines");
        assert_eq!(
            lines[0],
            "3.1                 COMPACT RINEX FORMAT                    CRINEX VERS   / TYPE",
        );

        assert_eq!(lines[1], "");
    }
}
