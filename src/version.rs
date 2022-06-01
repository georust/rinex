//! `RINEX` revision description and manipulation, 
//! contained in `header`

/// Current `RINEX` version supported to this day
pub const SUPPORTED_VERSION: Version = Version {
    major: 4,
    minor: 0
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Version {
    pub major: u8,
    pub minor: u8
}

impl Default for Version  {
    /// Builds a default `Version` object 
    fn default() -> Version {
        Version {
            major: 1,
            minor: 0
        }
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

mod test {
    #[test]
    fn test_version_object() {
        let version = super::Version::default();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
    }
    #[test]
    fn test_version_support() {
        let version = super::Version::default();
        assert_eq!(version.is_supported(), true);
        let version = super::SUPPORTED_VERSION; 
        assert_eq!(version.is_supported(), true);
    }
    #[test]
    fn test_version_non_support() {
        let version = super::Version::new(5, 0);
        assert_eq!(version.is_supported(), false);
    }
}
