//! Antenna Phase Center Variations
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown pcv code \"{0}\"")]
    UnknownPcv(String),
}

/// Antenna Phase Center Variation types
#[derive(Debug, Clone)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Pcv {
    /// Given data is absolute
    Absolute,
    /// Given data is relative, with type of relativity
    Relative(String),
}

impl Default for Pcv {
    fn default() -> Self {
        Self::Absolute
    }
}

impl std::str::FromStr for Pcv {
    type Err = Error;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if content.eq("A") {
            Ok(Self::Absolute)
        } else if content.eq("R") {
            Ok(Self::Relative("AOAD/M_T".to_string()))
        } else {
            Err(Error::UnknownPcv(content.to_string()))
        }
    }
}

impl Pcv {
    pub fn is_relative(&self) -> bool {
        match self {
            Self::Relative(_) => true,
            _ => false,
        }
    }
    pub fn is_absolute(&self) -> bool {
        !self.is_relative()
    }
    pub fn with_relative_type(&self, t: &str) -> Self {
        let mut s = self.clone();
        if s.is_relative() {
            s = Self::Relative(t.to_string())
        }
        s
    }
}
