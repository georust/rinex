use crate::constellation;
use crate::navigation;
use crate::navigation::{orbits::NAV_ORBITS, FrameClass, NavMsgType};
use crate::observable;
use crate::observable::Observable;
use crate::prelude::*;
use crate::sv;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown target \"{0}\"")]
    UnknownTarget(String),
    #[error("type guessing error \"{0}\"")]
    TypeGuessingError(String),
    #[error("expecting two epochs when describing a duration")]
    InvalidDuration,
    #[error("bad epoch description")]
    InvalidEpochDescription,
    #[error("bad elevation angle description")]
    InvalidElevationAngleDescription,
    #[error("bad azimuth angle description")]
    InvalidAzimuthAngleDescription,
    #[error("bad snr description")]
    InvalidSNRDescription,
    #[error("failed to parse sv")]
    SvParingError(#[from] sv::Error),
    #[error("constellation parsing error")]
    ConstellationParing(#[from] constellation::ParsingError),
    #[error("failed to parse epoch flag")]
    EpochFlagParsingError(#[from] crate::epoch::flag::Error),
    #[error("failed to parse constellation")]
    ConstellationParsingError,
    #[error("invalid nav item")]
    InvalidNavItem(#[from] crate::navigation::Error),
    #[error("observable parsing error")]
    ObservableParsing(#[from] observable::ParsingError),
    #[error("invalid duration description")]
    InvalidDurationItem(#[from] hifitime::Errors),
}

/// Target Item represents items that filters
/// or algorithms may target
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum TargetItem {
    /// Epoch Item
    EpochItem(Epoch),
    /// Duration Item
    DurationItem(Duration),
    /// Epoch Flag Item
    EpochFlagItem(EpochFlag),
    /// SNR value
    SnrItem(f64),
    /// Elevation Angle Item
    ElevationItem(f64),
    /// Azimuth Angle Item
    AzimuthItem(f64),
    /// List of Sv Item
    SvItem(Vec<Sv>),
    /// List of Constellation Item
    ConstellationItem(Vec<Constellation>),
    /// List of Observable Item
    ObservableItem(Vec<Observable>),
    /// List of Orbit fields item
    OrbitItem(Vec<String>),
    /// List of Navigation Messages
    NavMsgItem(Vec<NavMsgType>),
    /// List of Navigation Frame types
    NavFrameItem(Vec<FrameClass>),
    /// (Rx) ClockItem
    ClockItem,
}

impl std::ops::BitOrAssign for TargetItem {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.clone() | rhs;
    }
}

impl std::ops::BitOr for TargetItem {
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
            Self::ObservableItem(ref lhs) => match rhs {
                Self::ObservableItem(rhs) => {
                    let mut lhs = lhs.clone();
                    for r in rhs {
                        lhs.push(r);
                    }
                    Self::ObservableItem(lhs)
                },
                _ => self.clone(),
            },
            Self::OrbitItem(ref lhs) => match rhs {
                Self::OrbitItem(rhs) => {
                    let mut lhs = lhs.clone();
                    for r in rhs {
                        lhs.push(r);
                    }
                    Self::OrbitItem(lhs)
                },
                _ => self.clone(),
            },
            Self::NavMsgItem(ref lhs) => match rhs {
                Self::NavMsgItem(rhs) => {
                    let mut lhs = lhs.clone();
                    for r in rhs {
                        lhs.push(r);
                    }
                    Self::NavMsgItem(lhs)
                },
                _ => self.clone(),
            },
            Self::NavFrameItem(ref lhs) => match rhs {
                Self::NavFrameItem(rhs) => {
                    let mut lhs = lhs.clone();
                    for r in rhs {
                        lhs.push(r);
                    }
                    Self::NavFrameItem(lhs)
                },
                _ => self.clone(),
            },
            _ => self.clone(),
        }
    }
}

pub(crate) fn parse_sv_list(items: Vec<&str>) -> Result<Vec<Sv>, sv::Error> {
    let mut ret: Vec<Sv> = Vec::with_capacity(items.len());
    for item in items {
        let sv = Sv::from_str(item.trim())?;
        ret.push(sv);
    }
    Ok(ret)
}

pub(crate) fn parse_gnss_list(
    items: Vec<&str>,
) -> Result<Vec<Constellation>, constellation::ParsingError> {
    let mut ret: Vec<Constellation> = Vec::with_capacity(items.len());
    for item in items {
        let c = Constellation::from_str(item.trim())?;
        ret.push(c);
    }
    Ok(ret)
}

pub(crate) fn parse_obs_list(
    items: Vec<&str>,
) -> Result<Vec<Observable>, observable::ParsingError> {
    let mut ret: Vec<Observable> = Vec::with_capacity(items.len());
    for item in items {
        let obs = Observable::from_str(item.trim())?;
        ret.push(obs);
    }
    Ok(ret)
}

pub(crate) fn parse_nav_frames(items: Vec<&str>) -> Result<Vec<FrameClass>, navigation::Error> {
    let mut ret: Vec<FrameClass> = Vec::with_capacity(items.len());
    for item in items {
        let sv = FrameClass::from_str(item.trim())?;
        ret.push(sv);
    }
    Ok(ret)
}

pub(crate) fn parse_nav_msg(items: Vec<&str>) -> Result<Vec<NavMsgType>, navigation::Error> {
    let mut ret: Vec<NavMsgType> = Vec::with_capacity(items.len());
    for item in items {
        let msg = NavMsgType::from_str(item.trim())?;
        ret.push(msg);
    }
    Ok(ret)
}

pub(crate) fn parse_float_payload(content: &str) -> Result<f64, std::num::ParseFloatError> {
    f64::from_str(content.trim())
}

impl TargetItem {
    pub(crate) fn from_elevation(content: &str) -> Result<Self, Error> {
        if let Ok(float) = parse_float_payload(content) {
            Ok(Self::ElevationItem(float))
        } else {
            Err(Error::InvalidElevationAngleDescription)
        }
    }
    pub(crate) fn from_azimuth(content: &str) -> Result<Self, Error> {
        if let Ok(float) = parse_float_payload(content) {
            Ok(Self::AzimuthItem(float))
        } else {
            Err(Error::InvalidAzimuthAngleDescription)
        }
    }
    pub(crate) fn from_snr(content: &str) -> Result<Self, Error> {
        if let Ok(float) = parse_float_payload(content) {
            Ok(Self::SnrItem(float))
        } else {
            Err(Error::InvalidSNRDescription)
        }
    }
}

use itertools::Itertools;

impl std::str::FromStr for TargetItem {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        /*
         * Type guessing:
         * is used by Filter::from_str()
         * when operand comes first in description.
         * Otherwise, we muse use other methods
         */
        let items: Vec<&str> = c.split(",").collect();
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
                    Err(Error::InvalidEpochDescription)
                }
            } else {
                Err(Error::InvalidDuration)
            }
        /*
         * Sv
         */
        } else if let Ok(_sv) = Sv::from_str(items[0].trim()) {
            //TODO improve this:
            // do not test 1st entry only but all possible content
            Ok(Self::SvItem(parse_sv_list(items)?))
        /*
         * GNSS
         */
        } else if let Ok(_c) = Constellation::from_str(items[0].trim()) {
            //TODO improve this:
            // do not test 1st entry only but all possible content
            Ok(Self::ConstellationItem(parse_gnss_list(items)?))
        /*
         * Observables
         */
        } else if let Ok(_obs) = Observable::from_str(items[0].trim()) {
            //TODO improve this:
            // do not test 1st entry only but all possible content
            Ok(Self::ObservableItem(parse_obs_list(items)?))
        /*
         * Navigation Frames
         */
        } else if let Ok(_fr) = FrameClass::from_str(items[0].trim()) {
            //TODO improve this:
            // do not test 1st entry only but all possible content
            Ok(Self::NavFrameItem(parse_nav_frames(items)?))
        /*
         * Navigation Msg
         */
        } else if let Ok(_msg) = NavMsgType::from_str(items[0].trim()) {
            //TODO improve this:
            // do not test 1st entry only but all possible content
            Ok(Self::NavMsgItem(parse_nav_msg(items)?))
        } else {
            // try to match existing Orbit field(s).
            // To do so, we regroup all known Orbit fields, accross all NAV revisions,
            // and filter out non matching data fields.
            let matched_orbits: Vec<_> = NAV_ORBITS
                .iter()
                .flat_map(|entry| entry.items.clone())
                .map(|(key, _value)| key) // we're not interested in matching keys here
                .unique()
                .filter(|k| {
                    let mut found = false;
                    for item in &items {
                        // we use a lowercase comparison
                        // to make filter description case insensitive.
                        // Makes filter description much easier
                        found |= item.to_ascii_lowercase().eq(k);
                    }
                    found
                })
                .map(|s| s.to_string())
                .collect();

            if matched_orbits.len() > 0 {
                Ok(Self::OrbitItem(matched_orbits))
            } else {
                // not a single match
                Err(Error::TypeGuessingError(c.to_string()))
            }
        }
    }
}

impl From<Epoch> for TargetItem {
    fn from(e: Epoch) -> Self {
        Self::EpochItem(e)
    }
}

impl From<Duration> for TargetItem {
    fn from(dt: Duration) -> Self {
        Self::DurationItem(dt)
    }
}

impl From<EpochFlag> for TargetItem {
    fn from(f: EpochFlag) -> Self {
        Self::EpochFlagItem(f)
    }
}

impl From<Sv> for TargetItem {
    fn from(sv: Sv) -> Self {
        Self::SvItem(vec![sv])
    }
}

impl From<Vec<Sv>> for TargetItem {
    fn from(sv: Vec<Sv>) -> Self {
        Self::SvItem(sv.clone())
    }
}

impl From<Constellation> for TargetItem {
    fn from(c: Constellation) -> Self {
        Self::ConstellationItem(vec![c])
    }
}

impl From<Vec<Constellation>> for TargetItem {
    fn from(c: Vec<Constellation>) -> Self {
        Self::ConstellationItem(c.clone())
    }
}

impl From<NavMsgType> for TargetItem {
    fn from(msg: NavMsgType) -> Self {
        Self::NavMsgItem(vec![msg])
    }
}

impl From<FrameClass> for TargetItem {
    fn from(class: FrameClass) -> Self {
        Self::NavFrameItem(vec![class])
    }
}

impl From<Observable> for TargetItem {
    fn from(obs: Observable) -> Self {
        Self::ObservableItem(vec![obs])
    }
}

impl From<Vec<Observable>> for TargetItem {
    fn from(obs: Vec<Observable>) -> Self {
        Self::ObservableItem(obs.clone())
    }
}

impl std::fmt::Display for TargetItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ObservableItem(observables) => {
                write!(f, "physics: {:?}", observables)
            },
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
    use crate::Observable;
    use std::str::FromStr;
    #[test]
    fn algo_target_item() {
        let e = Epoch::default();
        let target: TargetItem = e.into();
        assert_eq!(target, TargetItem::EpochItem(e));

        let obs = Observable::default();
        let target: TargetItem = obs.clone().into();
        assert_eq!(target, TargetItem::ObservableItem(vec![obs.clone()]));
        assert_eq!(TargetItem::from_str("L1C").unwrap(), target);

        let msg = NavMsgType::LNAV;
        let target: TargetItem = msg.into();
        assert_eq!(target, TargetItem::NavMsgItem(vec![msg]));
        assert_eq!(TargetItem::from_str("LNAV").unwrap(), target);

        let fr = FrameClass::Ephemeris;
        let target: TargetItem = fr.into();
        assert_eq!(target, TargetItem::NavFrameItem(vec![fr]));
        assert_eq!(TargetItem::from_str("eph").unwrap(), target);

        assert_eq!(
            TargetItem::from_str("eph, ion").unwrap(),
            TargetItem::NavFrameItem(vec![FrameClass::Ephemeris, FrameClass::IonosphericModel])
        );

        assert_eq!(
            TargetItem::from_str("g08,g09,R03").unwrap(),
            TargetItem::SvItem(vec![
                Sv::from_str("G08").unwrap(),
                Sv::from_str("G09").unwrap(),
                Sv::from_str("R03").unwrap()
            ])
        );

        assert_eq!(
            TargetItem::from_str("GPS , BDS").unwrap(),
            TargetItem::ConstellationItem(vec![Constellation::GPS, Constellation::BeiDou])
        );

        let dt = Duration::from_str("1 d").unwrap();
        let target: TargetItem = dt.into();
        assert_eq!(target, TargetItem::DurationItem(dt));

        // test Matching NAV orbits
        for descriptor in vec![
            "iode",
            "crc",
            "crs",
            "health",
            "iode,crc,crs",
            "iode, crc, crs",
        ] {
            let target = TargetItem::from_str(descriptor);
            assert!(
                target.is_ok(),
                "failed to parse TargetItem::OrbitItem from \"{}\"",
                descriptor
            );
        }

        // test non matching NAV orbits
        for descriptor in vec!["oide", "ble", "blah, oide"] {
            let target = TargetItem::from_str(descriptor);
            assert!(
                target.is_err(),
                "false positive on TargetItem::OrbitItem from \"{}\", parsed {:?}",
                descriptor,
                target,
            );
        }
    }
    #[test]
    fn test_from_elevation() {
        let desc = " 1234  ";
        assert!(
            TargetItem::from_elevation(desc).is_ok(),
            "Failed to parse Elevation Target Item"
        );
    }
    #[test]
    fn test_from_azimuth() {
        let desc = " 12.34  ";
        assert!(
            TargetItem::from_azimuth(desc).is_ok(),
            "Failed to parse Azimuth Target Item"
        );
    }
    #[test]
    fn test_from_snr() {
        let desc = " 12.34  ";
        assert!(
            TargetItem::from_snr(desc).is_ok(),
            "Failed to parse SNR Target Item"
        );
    }
}
