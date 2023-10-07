use std::str::FromStr;

#[derive(PartialEq, Debug, Clone)]
pub enum Error {
    InvalidSnrCode,
}

/// `Snr` Signal to noise ratio description,
/// is attached to some observations
#[derive(Default, PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Snr {
    /// Snr ~= 0 dB/Hz
    DbHz0,
    /// Snr < 12 dB/Hz
    DbHz12,
    /// 12 dB/Hz <= Snr < 17 dB/Hz
    DbHz12_17,
    /// 18 dB/Hz <= Snr < 23 dB/Hz
    DbHz18_23,
    /// 24 dB/Hz <= Snr < 29 dB/Hz
    #[default]
    DbHz24_29,
    /// 30 dB/Hz <= Snr < 35 dB/Hz
    DbHz30_35,
    /// 36 dB/Hz <= Snr < 41 dB/Hz
    DbHz36_41,
    /// 42 dB/Hz <= Snr < 47 dB/Hz
    DbHz42_47,
    /// 48 dB/Hz <= Snr < 53 dB/Hz
    DbHz48_53,
    /// Snr >= 54 dB/Hz
    DbHz54,
}

impl std::fmt::LowerHex for Snr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let descriptor = match self {
            Self::DbHz0 => "0",
            Self::DbHz12 => "1",
            Self::DbHz12_17 => "2",
            Self::DbHz18_23 => "3",
            Self::DbHz24_29 => "4",
            Self::DbHz30_35 => "5",
            Self::DbHz36_41 => "6",
            Self::DbHz42_47 => "7",
            Self::DbHz48_53 => "8",
            Self::DbHz54 => "9",
        };
        f.write_str(descriptor)
    }
}

impl std::fmt::LowerExp for Snr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let descriptor = match self {
            Self::DbHz0 => "<< 12 dB",
            Self::DbHz12 => "< 12 dB",
            Self::DbHz12_17 => "[12, 17[ dB",
            Self::DbHz18_23 => "[18, 23[ dB",
            Self::DbHz24_29 => "[24, 29[ dB",
            Self::DbHz30_35 => "[30, 35[ dB",
            Self::DbHz36_41 => "[36, 41[ dB",
            Self::DbHz42_47 => "[42, 47[ dB",
            Self::DbHz48_53 => "[48, 53[ dB",
            Self::DbHz54 => "> 54 dB",
        };
        f.write_str(descriptor)
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
            "bad" => Ok(Snr::DbHz18_23),
            "weak" => Ok(Snr::DbHz24_29),
            "strong" => Ok(Snr::DbHz30_35),
            "excellent" => Ok(Snr::DbHz48_53),
            _ => Err(Error::InvalidSnrCode),
        }
    }
}

impl From<f64> for Snr {
    fn from(f_db: f64) -> Self {
        if f_db < 12.0 {
            Self::DbHz12
        } else if f_db <= 17.0 {
            Self::DbHz12_17
        } else if f_db <= 23.0 {
            Self::DbHz18_23
        } else if f_db <= 29.0 {
            Self::DbHz24_29
        } else if f_db <= 35.0 {
            Self::DbHz30_35
        } else if f_db <= 41.0 {
            Self::DbHz36_41
        } else if f_db <= 47.0 {
            Self::DbHz42_47
        } else if f_db <= 53.0 {
            Self::DbHz48_53
        } else {
            Self::DbHz54
        }
    }
}

impl From<Snr> for f64 {
    fn from(val: Snr) -> Self {
        match val {
            Snr::DbHz0 => 0.0_f64,
            Snr::DbHz12 => 12.0_f64,
            Snr::DbHz12_17 => 17.0_f64,
            Snr::DbHz18_23 => 23.0_f64,
            Snr::DbHz24_29 => 29.0_f64,
            Snr::DbHz30_35 => 35.0_f64,
            Snr::DbHz36_41 => 41.0_f64,
            Snr::DbHz42_47 => 47.0_f64,
            Snr::DbHz48_53 => 53.0_f64,
            Snr::DbHz54 => 54.0_f64,
        }
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
    use std::str::FromStr;
    #[test]
    fn observation_snr() {
        let snr = Snr::from_str("0").unwrap();
        assert_eq!(snr, Snr::DbHz0);
        assert!(snr.bad());

        let snr = Snr::from_str("9").unwrap();
        assert!(snr.excellent());

        let snr = Snr::from_str("10");
        assert!(snr.is_err());

        let snr: Snr = Snr::from(48_u8);
        assert_eq!(snr, Snr::DbHz48_53);
        assert!(snr.excellent());
        assert_eq!(format!("{:x}", snr), "8");
        assert_eq!(format!("{:e}", snr), "[48, 53[ dB");

        let snr: Snr = Snr::from(31.3);
        assert_eq!(snr, Snr::DbHz30_35);
        assert!(snr.strong());

        let snr: Snr = Snr::from(3.0);
        assert_eq!(snr, Snr::DbHz12);
        assert!(snr.bad());

        assert_eq!(Snr::from_str("excellent"), Ok(Snr::DbHz48_53));
        assert_eq!(Snr::from_str("strong"), Ok(Snr::DbHz30_35));
        assert_eq!(Snr::from_str("weak"), Ok(Snr::DbHz24_29));
        assert_eq!(Snr::from_str("bad"), Ok(Snr::DbHz18_23));

        assert!(Snr::from_str("bad").unwrap().bad());
        assert!(Snr::from_str("weak").unwrap().weak());
        assert!(Snr::from_str("strong").unwrap().strong());
        assert!(Snr::from_str("excellent").unwrap().excellent());
    }
}
