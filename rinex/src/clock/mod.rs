//! Clock RINEX module

mod clock;
pub use clock::{ClockProfile, ClockProfileType, Clock, ClockType};

use crate::version::Version;
use hifitime::TimeScale;
use std::str::FromStr;

use crate::prelude::DOMES;

/// Clock [RINEX] Record content
#[derive(Error, PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Entry {
    /// Epoch
    pub epoch: Epoch,
    /// Type of Clock
    pub clock_type: ClockType,
    /// Type of measurement
    pub profile_type: ClockProfileType,
    /// Measurement
    pub profile: ClockProfile,
}

pub(crate) fn is_new_entry(line: &str) -> bool {
    // first 2 bytes match a [ClockProfileType] code
    if line.len() < 3 {
        false
    } else {
        let content = &line[..2];
        ClockProfileType::from_str(content).is_ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::SV;
    use crate::version::Version;
    use std::str::FromStr;
    #[test]
    fn test_is_new_epoch() {
        let c = "AR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01";
        assert!(is_new_epoch(c));
        let c = "RA AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01";
        assert!(!is_new_epoch(c));
        let c = "DR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01";
        assert!(is_new_epoch(c));
        let c = "CR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01";
        assert!(is_new_epoch(c));
        let c = "AS AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01";
        assert!(is_new_epoch(c));
        let c = "CR USNO 1995 07 14 20 59 50.000000  2    0.123456789012E+00  -0.123456789012E-01";
        assert!(is_new_epoch(c));
        let c = "AS G16  1994 07 14 20 59  0.000000  2   -0.123456789012E+00 -0.123456789012E+01";
        assert!(is_new_epoch(c));
        let c = "A  G16  1994 07 14 20 59  0.000000  2   -0.123456789012E+00 -0.123456789012E+01";
        assert!(!is_new_epoch(c));
        let c = "z";
        assert!(!is_new_epoch(c));
    }
}