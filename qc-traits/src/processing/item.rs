use std::{num::ParseFloatError, str::FromStr};
use thiserror::Error;

use gnss_rs::{
    constellation::ParsingError as ConstellationParsingError,
    prelude::{Constellation, SV},
    sv::ParsingError as SVParsingError,
};

use hifitime::{Duration, Epoch, ParsingError as EpochParsingError};

#[derive(Debug, Error)]
pub enum ItemError {
    #[error("unknown filter item \"{0}\"")]
    UnknownItem(String),
    #[error("item guessing error: {0}")]
    TypeGuessingError(String),
    #[error("two valid epochs are required to describe a duration")]
    InvalidDuration,
    #[error("invalid epoch description")]
    InvalidEpoch,
    #[error("invalid SNR description")]
    InvalidSNR,
    #[error("invalid elevation angle (0 <= e <= 90)")]
    InvalidElevationAngle,
    #[error("invalid azimuth angle description (0 <= a <= 360)")]
    InvalidAzimuthAngle,
    #[error("invalid float number")]
    FloatParsing(#[from] ParseFloatError),
    #[error("sv item parsing")]
    SVParsing(#[from] SVParsingError),
    #[error("constellation item parsing")]
    ConstellationParing(#[from] ConstellationParsingError),
    #[error("duration item parsing")]
    InvalidDurationItem(#[from] EpochParsingError),
}

/// [FilterItem] represents items that filters or other
/// GNSS processing ops may apply to.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum FilterItem {
    /// Epoch Item
    EpochItem(Epoch),
    /// Duration Item
    DurationItem(Duration),
    /// SNR value, expressed in [dB]
    SNRItem(f64),
    /// Elevation Angle Item in degrees, 0 <= e <= 90°
    ElevationItem(f64),
    /// Azimuth Angle Item in degrees, 0 <= a <= 360°
    AzimuthItem(f64),
    /// List of spacecrafts described as [SV]
    SvItem(Vec<SV>),
    /// List of [Constellation]s
    ConstellationItem(Vec<Constellation>),
    /// Clock Offset Item
    ClockItem,
    /// List of complex items originally described as Strings
    ComplexItem(Vec<String>),
}

impl std::ops::BitOrAssign for FilterItem {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.clone() | rhs;
    }
}

impl std::ops::BitOr for FilterItem {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        match self {
            Self::SvItem(ref lhs) => match rhs {
                Self::SvItem(rhs) => {
                    let mut lhs = lhs.clone();
                    for r in rhs {
                        lhs.push(r);
                    }
                    Self::SvItem(lhs)
                },
                _ => self.clone(),
            },
            Self::ConstellationItem(ref lhs) => match rhs {
                Self::ConstellationItem(rhs) => {
                    let mut lhs = lhs.clone();
                    for r in rhs {
                        lhs.push(r);
                    }
                    Self::ConstellationItem(lhs)
                },
                _ => self.clone(),
            },
            _ => self.clone(),
        }
    }
}

pub(crate) fn parse_sv_list(items: Vec<&str>) -> Result<Vec<SV>, SVParsingError> {
    let mut ret: Vec<SV> = Vec::with_capacity(items.len());
    for item in items {
        let sv = SV::from_str(item.trim())?;
        ret.push(sv);
    }
    Ok(ret)
}

pub(crate) fn parse_gnss_list(
    items: Vec<&str>,
) -> Result<Vec<Constellation>, ConstellationParsingError> {
    let mut ret: Vec<Constellation> = Vec::with_capacity(items.len());
    for item in items {
        let c = Constellation::from_str(item.trim())?;
        ret.push(c);
    }
    Ok(ret)
}

fn parse_float_payload(content: &str) -> Result<f64, ParseFloatError> {
    f64::from_str(content.trim())
}

impl FilterItem {
    pub(crate) fn from_elevation(content: &str) -> Result<Self, ItemError> {
        if let Ok(float) = parse_float_payload(content) {
            if float >= 0.0 && float <= 90.0 {
                return Ok(Self::AzimuthItem(float));
            }
        }
        Err(ItemError::InvalidElevationAngle)
    }
    pub(crate) fn from_azimuth(content: &str) -> Result<Self, ItemError> {
        if let Ok(float) = parse_float_payload(content) {
            if float >= 0.0 && float <= 360.0 {
                return Ok(Self::AzimuthItem(float));
            }
        }
        Err(ItemError::InvalidAzimuthAngle)
    }
    pub(crate) fn from_snr(content: &str) -> Result<Self, ItemError> {
        if let Ok(float) = parse_float_payload(content) {
            Ok(Self::SNRItem(float))
        } else {
            Err(ItemError::InvalidSNR)
        }
    }
}

// use itertools::Itertools;

impl std::str::FromStr for FilterItem {
    type Err = ItemError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        /*
         * Type guessing
         */
        let c = content.trim();
        let items: Vec<&str> = c.split(',').collect();
        /*
         * Epoch and Durations
         */
        if let Ok(start) = Epoch::from_str(items[0].trim()) {
            if items.len() == 1 {
                Ok(Self::EpochItem(start))
            } else if items.len() == 2 {
                if let Ok(end) = Epoch::from_str(items[1].trim()) {
                    Ok(Self::DurationItem(end - start))
                } else {
                    Err(ItemError::InvalidEpoch)
                }
            } else {
                Err(ItemError::InvalidDuration)
            }
        /*
         * SV
         */
        } else if SV::from_str(items[0].trim()).is_ok() {
            //TODO improve this:
            // do not test 1st entry only but all possible content
            Ok(Self::SvItem(parse_sv_list(items)?))
        /*
         * GNSS Constellation
         */
        } else if Constellation::from_str(items[0].trim()).is_ok() {
            //TODO improve this:
            // do not test 1st entry only but all possible content
            Ok(Self::ConstellationItem(parse_gnss_list(items)?))
        } else {
            // define this item a "complex"
            Ok(Self::ComplexItem(
                items.iter().map(|s| s.to_string()).collect(),
            ))
        }
    }
}

impl From<Epoch> for FilterItem {
    fn from(e: Epoch) -> Self {
        Self::EpochItem(e)
    }
}

impl From<Duration> for FilterItem {
    fn from(dt: Duration) -> Self {
        Self::DurationItem(dt)
    }
}

impl From<SV> for FilterItem {
    fn from(sv: SV) -> Self {
        Self::SvItem(vec![sv])
    }
}

impl From<Vec<SV>> for FilterItem {
    fn from(sv: Vec<SV>) -> Self {
        Self::SvItem(sv.clone())
    }
}

impl From<Constellation> for FilterItem {
    fn from(c: Constellation) -> Self {
        Self::ConstellationItem(vec![c])
    }
}

impl From<Vec<Constellation>> for FilterItem {
    fn from(c: Vec<Constellation>) -> Self {
        Self::ConstellationItem(c.clone())
    }
}

impl std::fmt::Display for FilterItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ConstellationItem(gnss) => {
                write!(f, "gnss: {:?}", gnss)
            },
            Self::SvItem(svs) => {
                write!(f, "sv: {:?}", svs)
            },
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use gnss_rs::prelude::{Constellation, SV};
    use std::str::FromStr;
    #[test]
    fn algo_target_item() {
        let e = Epoch::default();
        let target: FilterItem = e.into();
        assert_eq!(target, FilterItem::EpochItem(e));

        assert_eq!(
            FilterItem::from_str("g08,g09,R03").unwrap(),
            FilterItem::SvItem(vec![
                SV::from_str("G08").unwrap(),
                SV::from_str("G09").unwrap(),
                SV::from_str("R03").unwrap()
            ])
        );

        assert_eq!(
            FilterItem::from_str("GPS , BDS").unwrap(),
            FilterItem::ConstellationItem(vec![Constellation::GPS, Constellation::BeiDou])
        );

        let dt = Duration::from_str("1 d").unwrap();
        let target: FilterItem = dt.into();
        assert_eq!(target, FilterItem::DurationItem(dt));
    }
    #[test]
    fn test_from_elevation() {
        let desc = "90";
        assert!(
            FilterItem::from_elevation(desc).is_ok(),
            "Failed to parse Elevation Target Item"
        );
    }
    #[test]
    fn test_from_azimuth() {
        let desc = " 12.34  ";
        assert!(
            FilterItem::from_azimuth(desc).is_ok(),
            "Failed to parse Azimuth Target Item"
        );
    }
    #[test]
    fn test_from_snr() {
        let desc = " 12.34  ";
        assert!(
            FilterItem::from_snr(desc).is_ok(),
            "Failed to parse SNR Target Item"
        );
    }
}
