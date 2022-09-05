//! `RINEX` revision description and manipulation, 
//! contained in `header`

/// Current `RINEX` version supported to this day
pub const SUPPORTED_VERSION: Version = Version {
    major: 4,
    minor: 0
};

#[derive(Copy, Clone, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Version {
    pub major: u8,
    pub minor: u8,
}

impl Default for Version  {
    /// Builds a default `Version` object 
    fn default() -> Version {
        SUPPORTED_VERSION
    }
}

impl std::str::FromStr for Version {
    type Err = std::num::ParseIntError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        match s.contains(".") {
            true => {
                let digits: Vec<&str> = s.split(".")
                    .collect();
                Ok(Version {
                    major: u8::from_str_radix(digits.get(0)
                        .unwrap(), 10)?,
                    minor: u8::from_str_radix(digits.get(1)
                        .unwrap(), 10)?,
                })
            },
            false => {
                Ok(Version {
                    major: u8::from_str_radix(s,10)?,
                    minor: 0
                })
            }
        }
    }
}

impl Version {
    /// Builds a new `Version` object
    pub fn new (major: u8, minor: u8) -> Version { Version { major, minor }}
    /// Returns true if this version is supported
    pub fn is_supported (&self) -> bool {
        if self.major < SUPPORTED_VERSION.major {
            true
        } else if self.major == SUPPORTED_VERSION.major {
            self.minor <= SUPPORTED_VERSION.minor
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn version() {
        let version = Version::default();
        assert_eq!(version.major, SUPPORTED_VERSION.major);
        assert_eq!(version.minor, SUPPORTED_VERSION.minor);
        
        let version = Version::from_str("1");
        assert_eq!(version.is_ok(), true);
        let version = version.unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        
        let version = Version::from_str("1.2");
        assert_eq!(version.is_ok(), true);
        let version = version.unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        
        let version = Version::from_str("3.02");
        assert_eq!(version.is_ok(), true);
        let version = version.unwrap();
        assert_eq!(version.major, 3);
        assert_eq!(version.minor, 2);
        
        let version = Version::from_str("a.b");
        assert_eq!(version.is_err(), true);
    }
    #[test]
    fn supported_version() {
        let version = Version::default();
        assert_eq!(version.is_supported(), true);
        let version = SUPPORTED_VERSION; 
        assert_eq!(version.is_supported(), true);
    }
    #[test]
    fn non_supported_version() {
        let version = Version::new(5, 0);
        assert_eq!(version.is_supported(), false);
    }
    #[test]
    fn test_comparison() {
        let v_a = Version::from_str("1.2").unwrap();
        let v_b = Version::from_str("3.02").unwrap();
        assert_eq!(v_b > v_a, true);
        assert_eq!(v_b == v_a, false);
    }
}
