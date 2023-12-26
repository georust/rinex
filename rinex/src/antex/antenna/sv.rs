use gnss_rs::prelude::SV;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone, Error)]
pub enum SvAntennaParsingError {
    #[error("cospar bad length")]
    CosparBadLength,
    #[error("failed to parse cospar launch year")]
    CosparLaunchYearParsing,
    #[error("failed to parse cospar launch code")]
    CosparLaunchCodeParsing,
}

#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct SvAntenna {
    /// IGS antenna code
    pub igs_type: String,
    /// Spacecraft to which this antenna is attached to
    pub sv: SV,
    /// Cospar information
    pub cospar: Cospar,
}

#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Cospar {
    /// Vehicle launch year
    pub launch_year: u16,
    /// Launcher ID
    pub launch_vehicle: String,
    /// Launch code
    pub launch_code: char,
}

impl std::str::FromStr for Cospar {
    type Err = SvAntennaParsingError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let s = content.trim();
        if s.len() != 9 {
            return Err(SvAntennaParsingError::CosparBadLength);
        }
        let year = s[0..4]
            .parse::<u16>()
            .map_err(|_| SvAntennaParsingError::CosparLaunchYearParsing)?;

        let _launch_code = s[8..9]
            .chars()
            .next()
            .ok_or(SvAntennaParsingError::CosparLaunchCodeParsing)?;

        Ok(Self {
            launch_year: year,
            launch_vehicle: s[4..6].to_string(),
            launch_code: s[8..9].chars().next().unwrap(),
        })
    }
}
