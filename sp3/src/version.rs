//! sp3 version

use crate::ParsingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Version {
    A,
    B,
    /// SP3-C defined in [Bibliography::SP3cRev]
    C,
    #[default]
    /// SP3-D defined in [Bibliography::SP3dRev]
    D,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::A => f.write_str("a"),
            Self::B => f.write_str("b"),
            Self::C => f.write_str("c"),
            Self::D => f.write_str("d"),
        }
    }
}

impl std::str::FromStr for Version {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("a") {
            Ok(Self::A)
        } else if s.eq("b") {
            Ok(Self::B)
        } else if s.eq("c") {
            Ok(Self::C)
        } else if s.eq("d") {
            Ok(Self::D)
        } else {
            Err(ParsingError::UnknownVersion(s.to_string()))
        }
    }
}

impl From<Version> for u8 {
    fn from(val: Version) -> Self {
        match val {
            Version::A => 1,
            Version::B => 2,
            Version::C => 3,
            Version::D => 4,
        }
    }
}

impl From<u8> for Version {
    fn from(lhs: u8) -> Version {
        match lhs {
            4..=u8::MAX => Version::D,
            0..=3 => Version::C,
        }
    }
}

impl std::ops::Add<u8> for Version {
    type Output = Self;
    fn add(self, rhs: u8) -> Self {
        let s: u8 = self.into();
        (s + rhs).into()
    }
}

impl std::ops::Sub<u8> for Version {
    type Output = Self;
    fn sub(self, rhs: u8) -> Self {
        let s: u8 = self.into();
        (s - rhs).into()
    }
}

#[cfg(test)]
mod test {
    use super::Version;
    use std::str::FromStr;
    #[test]
    fn version() {
        for (desc, expected) in vec![("c", Version::C), ("d", Version::D)] {
            assert!(
                Version::from_str(desc).is_ok(),
                "failed to parse Version from \"{}\"",
                desc
            );
        }

        for (vers, expected) in vec![(Version::C, 3), (Version::D, 4)] {
            let version: u8 = vers.into();
            assert_eq!(version, expected, "convertion to integer failed");
        }

        assert!(Version::C < Version::D);
        assert!(Version::D >= Version::C);

        let version: Version = 4_u8.into();
        assert_eq!(version, Version::D);
        assert_eq!(version + 1, Version::D);
        assert_eq!(version - 1, Version::C);

        let version: Version = 3_u8.into();
        assert_eq!(version, Version::C);
        assert_eq!(version + 1, Version::D);
        assert_eq!(version - 1, Version::C);

        assert!(Version::A < Version::B);
        assert!(Version::A < Version::C);
        assert!(Version::A < Version::D);
        assert!(Version::D > Version::C);
    }
}
