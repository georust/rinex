use bitflags::bitflags;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

//use crate::{};

#[derive(Debug, Clone)]
pub enum Error {
	InvalidSnrCode,
}

/// `Snr` Signal to noise ratio description,
/// is attached to some observations
#[repr(u8)]
#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Snr {
    /// Snr ~= 0 dB/Hz
    DbHz0 = 0,
    /// Snr < 12 dB/Hz
    DbHz12 = 1,
    /// 12 dB/Hz <= Snr < 17 dB/Hz
    DbHz12_17 = 2,
    /// 18 dB/Hz <= Snr < 23 dB/Hz
    DbHz18_23 = 3,
    /// 24 dB/Hz <= Snr < 29 dB/Hz
    DbHz24_29 = 4,
    /// 30 dB/Hz <= Snr < 35 dB/Hz
    DbHz30_35 = 5,
    /// 36 dB/Hz <= Snr < 41 dB/Hz
    DbHz36_41 = 6,
    /// 42 dB/Hz <= Snr < 47 dB/Hz
    DbHz42_47 = 7,
    /// 48 dB/Hz <= Snr < 53 dB/Hz
    DbHz48_53 = 8,
    /// Snr >= 54 dB/Hz
    DbHz54 = 9,
}

impl std::fmt::Display for Snr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DbHz0 => "0".fmt(f),
            Self::DbHz12 => "1".fmt(f),
            Self::DbHz12_17 => "2".fmt(f),
            Self::DbHz18_23 => "3".fmt(f),
            Self::DbHz24_29 => "4".fmt(f),
            Self::DbHz30_35 => "5".fmt(f),
            Self::DbHz36_41 => "6".fmt(f),
            Self::DbHz42_47 => "7".fmt(f),
            Self::DbHz48_53 => "8".fmt(f),
            Self::DbHz54 => "9".fmt(f),
        }
    }
}

impl FromStr for Snr {
    type Err = Error;
    fn from_str(code: &str) -> Result<Self, Self::Err> {
        match code.trim() {
            "0" => Ok(Snr::DbHz0),
            "1" => Ok(Snr::DbHz12),
            "2" => Ok(Snr::DbHz12_17),
            "3" => Ok(Snr::DbHz18_23),
            "4" => Ok(Snr::DbHz24_29),
            "5" => Ok(Snr::DbHz30_35),
            "6" => Ok(Snr::DbHz36_41),
            "7" => Ok(Snr::DbHz42_47),
            "8" => Ok(Snr::DbHz48_53),
            "9" => Ok(Snr::DbHz54),
            _ => Err(Error::InvalidSnrCode)
        }
    }
}

impl Default for Snr {
    fn default() -> Snr {
        Snr::DbHz24_29
    }
}

impl From<f64> for Snr {
	fn from(f: f64) -> Self {
		Self::from(f as u8)
	}
}

impl From<u8> for Snr {
	fn from(u: u8) -> Self {
		if u >= 54 {
			Self::DbHz54
		} else if u >= 48 {
			Self::DbHz48_53
		} else if u >= 42 {
			Self::DbHz42_47
		} else if u >= 36 {
			Self::DbHz36_41
		} else if u >= 30 {
			Self::DbHz30_35
		} else if u >= 24 {
			Self::DbHz24_29
		} else if u >= 18 {
			Self::DbHz18_23
		} else if u >= 12 {
			Self::DbHz12_17
		} else if u > 1 {
			Self::DbHz12
		} else {
			Self::DbHz0
		}
	}
}

impl Snr {
	pub fn new(quality: &str) -> Self {
		match quality.trim() {
			"excellent" => Self::DbHz42_47,
			"strong" => Self::DbHz30_35,
			"weak" => Self::DbHz24_29,
			_ => Self::DbHz18_23,
		}
	}
	/// Returns true if self describes a bad signal level
    pub fn bad(self) -> bool {
        self <= Snr::DbHz18_23
    }
    /// Returns true if `self` describes a weak signal level
    pub fn weak(self) -> bool {
        self < Snr::DbHz30_35
    }
    /// Returns true if `self` describes a strong signal level, defined in standard specifications
    pub fn strong(self) -> bool {
        self >= Snr::DbHz30_35
    }
    /// Returns true if `self` is a very strong signal level
    pub fn excellent(self) -> bool {
        self > Snr::DbHz42_47
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn observation_snr() {
        let snr = Snr::from_str("0")
			.unwrap();
        assert_eq!(snr, Snr::DbHz0);
        assert!(snr.bad());

        let snr = Snr::from_str("9")
			.unwrap();
		assert!(snr.excellent());

        let snr = Snr::from_str("10");
		assert!(snr.is_err());
		
		let snr: Snr = Snr::from(48_u8);
		assert_eq!(snr, Snr::DbHz48_53);
		assert!(snr.excellent());

		let snr: Snr = Snr::from(31.3);
		assert_eq!(snr, Snr::DbHz30_35);
		assert!(snr.strong());

		let snr: Snr = Snr::from(3.0);
		assert_eq!(snr, Snr::DbHz12);
		assert!(snr.bad());
		
		assert_eq!(Snr::new("excellent"), Snr::DbHz42_47);
		assert_eq!(Snr::new("strong"), Snr::DbHz30_35);
		assert_eq!(Snr::new("weak"), Snr::DbHz24_29);
		assert_eq!(Snr::new("bad"), Snr::DbHz18_23);
    }
}
