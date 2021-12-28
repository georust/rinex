//! This modules contains the supported `RINEX` Version, 
//! aswell as a set of macros to manipulate and describe
//! RINEX File versionning

use std::str::FromStr;

/// Current `RINEX` version supported to this day
pub const SUPPORTED_VERSION: Version = Version {
    major: 4,
    minor: 0
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Version {
    major: u8,
    minor: u8
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
    /// Returns version major #
    pub fn get_major (&self) -> u8 { self.major }
    /// Returns version minor #
    pub fn get_minor (&self) -> u8 { self.major }
    
    /// Returns true if this version is supported
    pub fn is_supported (&self) -> bool {
        if self.major < SUPPORTED_VERSION.major {
            true
        } else {
            self.minor <= SUPPORTED_VERSION.minor
        }
    }
}

mod test {
    use super::*;
    #[test]
    fn test_version_object() {
        let version = Version::default();
        assert_eq!(version.get_major(), 1);
        assert_eq!(version.get_minor(), 1);
    }
    fn test_version_support() {
        let version = Version::default();
        assert_eq!(version.is_supported(), true);
        let version = SUPPORTED_VERSION; 
        assert_eq!(version.is_supported(), true);
    }
}
