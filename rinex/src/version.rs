//! `RINEX` revision description
use thiserror::Error;

/// Current `RINEX` version supported to this day
pub const SUPPORTED_VERSION: Version = Version { major: 4, minor: 0 };

/// Version is used to describe RINEX standards revisions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Version {
    /// Version major number
    pub major: u8,
    /// Version minor number
    pub minor: u8,
}

#[derive(Clone, Debug, Error)]
pub enum ParsingError {
    #[error("non supported version \"{0}\"")]
    NotSupported(String),
    #[error("failed to parse version")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl Default for Version {
    /// Builds a default `Version` object
    fn default() -> Self {
        SUPPORTED_VERSION
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl std::ops::Add<u8> for Version {
    type Output = Version;
    fn add(self, major: u8) -> Version {
        Version {
            major: self.major + major,
            minor: self.minor,
        }
    }
}

impl std::ops::AddAssign<u8> for Version {
    fn add_assign(&mut self, major: u8) {
        self.major += major;
    }
}

impl std::ops::Sub<u8> for Version {
    type Output = Version;
    fn sub(self, major: u8) -> Version {
        if major >= self.major {
            // clamp @ V1.X
            Version {
                major: 1,
                minor: self.minor,
            }
        } else {
            Version {
                major: self.major - major,
                minor: self.minor,
            }
        }
    }
}

impl std::ops::SubAssign<u8> for Version {
    fn sub_assign(&mut self, major: u8) {
        if major >= self.major {
            // clamp @ V1.X
            self.major = 1;
        } else {
            self.major -= major;
        }
    }
}

impl From<Version> for (u8, u8) {
    fn from(v: Version) -> Self {
        (v.major, v.minor)
    }
}

impl std::str::FromStr for Version {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.contains('.') {
            true => {
                let mut digits = s.split('.');
                Ok(Self {
                    major: digits.next().unwrap().parse::<u8>()?,
                    minor: digits.next().unwrap().parse::<u8>()?,
                })
            },
            false => Ok(Self {
                major: s.parse::<u8>()?,
                minor: 0,
            }),
        }
    }
}

impl Version {
    /// Builds a new `Version` object
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }
    /// Returns true if this version is supported
    pub fn is_supported(&self) -> bool {
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
        assert!(version.is_ok());
        let version = version.unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);

        let version = Version::from_str("1.2");
        assert!(version.is_ok());
        let version = version.unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);

        let version = Version::from_str("3.02");
        assert!(version.is_ok());
        let version = version.unwrap();
        assert_eq!(version.major, 3);
        assert_eq!(version.minor, 2);

        let version = Version::from_str("a.b");
        assert!(version.is_err());
    }
    #[test]
    fn supported_version() {
        let version = Version::default();
        assert!(version.is_supported());
        let version = SUPPORTED_VERSION;
        assert!(version.is_supported());
    }
    #[test]
    fn non_supported_version() {
        let version = Version::new(5, 0);
        assert!(!version.is_supported());
    }
    #[test]
    fn version_comparison() {
        let v_a = Version::from_str("1.2").unwrap();
        let v_b = Version::from_str("3.02").unwrap();
        assert!(v_b > v_a);
        assert!(v_b != v_a);
    }
    #[test]
    fn version_arithmetics() {
        let version = Version::new(3, 2);
        assert_eq!(version + 1, Version::new(4, 2));
        assert_eq!(version + 2, Version::new(5, 2));
        assert_eq!(version - 2, Version::new(1, 2));
        assert_eq!(version - 3, Version::new(1, 2)); // clamped

        let (maj, min): (u8, u8) = version.into();
        assert_eq!(maj, 3);
        assert_eq!(min, 2);
    }
}
