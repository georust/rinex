use thiserror::Error;
use crate::sv;
use crate::observable;
use crate::navigation;
use crate::constellation;
use crate::prelude::*;
use crate::observable::Observable;
use crate::navigation::{FrameClass, MsgType};
use std::str::FromStr;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown target \"{0}\"")]
    UnknownTarget(String),
	#[error("expecting two epochs when describing a duration")]
	InvalidDuration,
	#[error("bad epoch description")]
	InvalidEpochDescription,
	#[error("bad elevation range description")]
	InvalidElevationRangeDescription,
    #[error("failed to parse sv")]
    SvParingError(#[from] sv::Error),
    #[error("failed to parse constellation")]
    ConstellationParingError(#[from] constellation::Error),
    #[error("failed to parse (float) payload")]
    //FilterParsingError(#[from] std::num::ParseFloatError),
	ParseFloatItemError,
    #[error("failed to parse epoch flag")]
    EpochFlagParsingError(#[from] crate::epoch::flag::Error),
    #[error("failed to parse constellation")]
    ConstellationParsingError,
    #[error("invalid nav item")]
    InvalidNavItem(#[from] crate::navigation::record::Error),
    #[error("invalid observable item")]
    InvalidObsItem(#[from] crate::observable::Error),
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
	/// SNR Range 
	SnrRangeItem((f64, f64)),
    /// Elevation Angle Item
    ElevationItem(f64),
    /// Elevation Range Item
    ElevationRangeItem((f64, f64)),
    /// Azimuth Angle Item
    AzimuthItem(f64),
    /// Azimuth Range Item
    AzimuthRangeItem((f64, f64)),
    /// List of Sv Item
    SvItem(Vec<Sv>),
    /// List of Constellation Item
    ConstellationItem(Vec<Constellation>),
    /// List of Observable Item
    ObservableItem(Vec<Observable>),
    /// List of Orbit fields item
    OrbitItem(Vec<String>),
    /// List of Navigation Messages
    NavMsgItem(Vec<MsgType>),
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

fn parse_sv_list(items: Vec<&str>) -> Result<Vec<Sv>, sv::Error> {
	let mut ret: Vec<Sv> = Vec::with_capacity(items.len());
	for item in items {
		let sv = Sv::from_str(item.trim())?;
		ret.push(sv);
	}
	Ok(ret)
}

fn parse_gnss_list(items: Vec<&str>) -> Result<Vec<Constellation>, constellation::Error> {
	let mut ret: Vec<Constellation> = Vec::with_capacity(items.len());
	for item in items {
		let c = Constellation::from_str(item.trim())?;
		ret.push(c);
	}
	Ok(ret)
}

fn parse_obs_list(items: Vec<&str>) -> Result<Vec<Observable>, observable::Error> {
	let mut ret: Vec<Observable> = Vec::with_capacity(items.len());
	for item in items {
		let obs = Observable::from_str(item.trim())?;
		ret.push(obs);
	}
	Ok(ret)
}

fn parse_nav_frames(items: Vec<&str>) -> Result<Vec<FrameClass>, navigation::record::Error> {
	let mut ret: Vec<FrameClass> = Vec::with_capacity(items.len());
	for item in items {
		let sv = FrameClass::from_str(item.trim())?;
		ret.push(sv);
	}
	Ok(ret)
}

fn parse_nav_msg(items: Vec<&str>) -> Result<Vec<MsgType>, navigation::record::Error> {
	let mut ret: Vec<MsgType> = Vec::with_capacity(items.len());
	for item in items {
		let msg = MsgType::from_str(item.trim())?;
		ret.push(msg);
	}
	Ok(ret)
}

fn parse_float_payload(item: &str) -> Result<(f64, Option<f64>), std::num::ParseFloatError> {
	let items: Vec<&str> = item.trim().split(",").collect();
	if items.len() >= 2 {
		let f1 = f64::from_str(items[0].trim())?;
		let f2 = f64::from_str(items[1].trim())?;
		Ok((f1, Some(f2)))
	} else { 
		let f1 = f64::from_str(items[0].trim())?;
		Ok((f1, None))
	}
}

impl std::str::FromStr for TargetItem {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        if c.starts_with("snr:") {
            match parse_float_payload(&c[4..]) {
                Ok((s1, None)) => Ok(Self::SnrItem(s1)),
                Ok((s1, Some(s2))) => Ok(Self::SnrRangeItem((s1,s2))),
                _ => Err(Error::ParseFloatItemError)
            }
        } else if c.starts_with("elev:") {
            match parse_float_payload(&c[5..]) {
                Ok((s1, None)) => Ok(Self::SnrItem(s1)),
                Ok((s1, Some(s2))) => Ok(Self::SnrRangeItem((s1,s2))),
                _ => Err(Error::ParseFloatItemError)
            }
        } else if c.starts_with("clk") {
            Ok(Self::ClockItem)
        } else {
            /* type guessing */
			let items: Vec<&str> = c.split(",")
				.collect();
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
			} else if let Ok(sv) = Sv::from_str(items[0].trim()) {
				Ok(Self::SvItem(parse_sv_list(items)?))
			/*
			 * GNSS
			 */
			} else if let Ok(c) = Constellation::from_str(items[0].trim()) {
				Ok(Self::ConstellationItem(parse_gnss_list(items)?))
			/*
			 * Observables
			 */
			} else if let Ok(obs) = Observable::from_str(items[0].trim()) {
				Ok(Self::ObservableItem(parse_obs_list(items)?))
			/* 
			 * Navigation Frames 
			 */
			} else if let Ok(fr) = FrameClass::from_str(items[0].trim()) {
				Ok(Self::NavFrameItem(parse_nav_frames(items)?))
			/* 
			 * Navigation Msg
			 */
			} else if let Ok(msg) = MsgType::from_str(items[0].trim()) {
				Ok(Self::NavMsgItem(parse_nav_msg(items)?))
			/*
			 * Elevation Angle 
			 */
			} else if let Ok(e1) = f64::from_str(items[0].trim()) {
				if items.len() == 1 {
					Ok(Self::ElevationItem(e1))
				} else if items.len() == 2 {
					if let Ok(e2) = f64::from_str(items[1].trim()) {
						Ok(Self::ElevationRangeItem((e1, e2)))
					} else {
						Err(Error::InvalidElevationRangeDescription)
					}
				} else {
					Err(Error::InvalidElevationRangeDescription)
				}
            } else {
                Err(Error::UnknownTarget(c.to_string()))
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

impl From<MsgType> for TargetItem {
    fn from(msg: MsgType) -> Self {
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
			_ => Ok(())
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

        /*let f = EpochFlag::default();
        let target: TargetItem = f.into();
        assert_eq!(target, TargetItem::EpochFlagItem(f));
        assert_eq!(TargetItem::from_str("f:0").unwrap(), target);*/

        let obs = Observable::default();
        let target: TargetItem = obs.clone().into();
        assert_eq!(target, TargetItem::ObservableItem(vec![obs.clone()]));
        assert_eq!(TargetItem::from_str("L1C").unwrap(), target);

        let msg = MsgType::LNAV;
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

		let snr = TargetItem::from_str("snr:10");
		assert!(snr.is_ok());
		let snr = TargetItem::from_str("snr:10,12");
		assert!(snr.is_ok());
		
		let elev = TargetItem::from_str("elev:30");
		assert!(elev.is_ok());
		let elev = TargetItem::from_str("elev:30,38");
		assert!(elev.is_ok());
    }
}
