mod sampling;
pub use sampling::Decimation;

mod mask;
pub use mask::{Mask, MaskFilter, MaskOperand};

//mod processing;
//pub use processing::{Processing, AverageType};

use thiserror::Error;
use crate::navigation::{MsgType, FrameClass};
use crate::{Epoch, Duration, EpochFlag, Sv, Constellation, Observable};

/// Target Item represents items that filter operations
/// or algorithms may target
#[derive(Clone, Debug, PartialEq)]
pub enum TargetItem {
    /// Epoch Item
    EpochItem(Epoch),
    /// Duration Item
    DurationItem(Duration),
    /// Epoch Flag Item
    EpochFlagItem(EpochFlag),
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
    NavMsgItem(Vec<MsgType>),
    /// List of Navigation Frame types
    NavFrameItem(Vec<FrameClass>),
}

impl std::str::FromStr for TargetItem {
    type Err = AlgorithmError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        /*
         * native parsing: epoch
         */
        if let Ok(epoch) = Epoch::from_str(c) {
            Ok(Self::EpochItem(epoch))
        } else {
            /*
             * complex descriptor
             */
            if c.starts_with("dt:") { // Duration description
                let duration = Duration::from_str(&c[3..])?;
                Ok(Self::DurationItem(duration))

            } else if c.starts_with("f:") { // Epoch Flag description
                let flag = EpochFlag::from_str(&c[2..])?;
                Ok(Self::EpochFlagItem(flag))

            } else if c.starts_with("elev:") { // Elevation Angle description
                let angle = f64::from_str(&c[5..])?;
                Ok(Self::ElevationItem(angle))

            } else if c.starts_with("azi:") { // Azimuth angle description
                let angle = f64::from_str(&c[4..])?;
                Ok(Self::ElevationItem(angle))

            } else if c.starts_with("sv:") {
                let items: Vec<&str> = c[3..].split(",").collect();
                let mut svs: Vec<Sv> = Vec::with_capacity(items.len());
                for item in items {
                    let sv = Sv::from_str(item.trim())?;
                    svs.push(sv);
                }
				Ok(Self::SvItem(svs))
            
            } else if c.starts_with("obs:") {
                let items: Vec<&str> = c[4..].split(",").collect();
                let mut obss: Vec<Observable> = Vec::with_capacity(items.len());
                for item in items {
                    let obs = Observable::from_str(item.trim())?;
                    obss.push(obs);
                }
				Ok(Self::ObservableItem(obss))
            
            } else if c.starts_with("orb:") {
                let items: Vec<&str> = c[4..].split(",").collect();
                let mut orbs: Vec<String> = Vec::with_capacity(items.len());
                for item in items {
                    let orb = item.trim().to_string();
                    orbs.push(orb);
                }
				Ok(Self::OrbitItem(orbs))
            
            } else if c.starts_with("gnss:") {
                let items: Vec<&str> = c[5..].split(",").collect();
                let mut gnss: Vec<Constellation> = Vec::with_capacity(items.len());
                for item in items {
                    let c = Constellation::from_str(item.trim())?;
                    gnss.push(c);
                }
				Ok(Self::ConstellationItem(gnss))
            
            } else if c.starts_with("nav:fr:") {
                let items: Vec<&str> = c[7..].split(",").collect();
                let mut fr: Vec<FrameClass> = Vec::with_capacity(items.len());
                for item in items {
                    let f = FrameClass::from_str(item.trim())?;
                    fr.push(f);
                }
				Ok(Self::NavFrameItem(fr))
            
            } else if c.starts_with("nav:msg:") {
                let items: Vec<&str> = c[8..].split(",").collect();
                let mut msg: Vec<MsgType> = Vec::with_capacity(items.len());
                for item in items {
                    let m = MsgType::from_str(item.trim())?;
                    msg.push(m);
                }
				Ok(Self::NavMsgItem(msg))
            
            } else {
                Err(AlgorithmError::UnrecognizedTarget)
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

impl From<Constellation> for TargetItem {
    fn from(c: Constellation) -> Self {
        Self::ConstellationItem(vec![c])
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

#[derive(Debug, Error)]
pub enum AlgorithmError {
    #[error("unrecognized operand")]
    UnknownOperand,
    #[error("unrecognized target")]
    UnrecognizedTarget,
    #[error("malformed target descriptor")]
    MalformedDescriptor,
    #[error("failed to parse (float) filter value")]
    FilterParsingError(#[from] std::num::ParseFloatError),
    #[error("failed to parse epoch flag")]
    EpochFlagParsingError(#[from] crate::epoch::flag::Error),
    #[error("failed to parse constellation")]
    ConstellationParsingError(#[from] crate::constellation::Error),
    #[error("failed to parse sv")]
    SvParsingError(#[from] crate::sv::Error),
    #[error("invalid nav item")]
    InvalidNavItem(#[from] crate::navigation::record::Error),
    #[error("invalid observable item")]
    InvalidObsItem(#[from] crate::observable::Error),
    #[error("invalid duration description")]
    InvalidDurationItem(#[from] hifitime::Errors),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Observable;
    use std::str::FromStr;
    #[test]
    fn test_target_item() {
        let e = Epoch::default();
        let target: TargetItem = e.into();
        assert_eq!(target, TargetItem::EpochItem(e));

        let f = EpochFlag::default();
        let target: TargetItem = f.into();
        assert_eq!(target, TargetItem::EpochFlagItem(f));
        assert_eq!(TargetItem::from_str("f:0").unwrap(), target);
        
        let obs = Observable::default();
        let target: TargetItem = obs.clone().into();
        assert_eq!(target, TargetItem::ObservableItem(vec![obs.clone()]));
        assert_eq!(TargetItem::from_str("obs:L1C").unwrap(), target);
        
        let msg = MsgType::LNAV;
        let target: TargetItem = msg.into();
        assert_eq!(target, TargetItem::NavMsgItem(vec![msg]));
        assert_eq!(TargetItem::from_str("nav:msg:LNAV").unwrap(), target);
        
        let fr = FrameClass::Ephemeris;
        let target: TargetItem = fr.into();
        assert_eq!(target, TargetItem::NavFrameItem(vec![fr]));
        assert_eq!(TargetItem::from_str("nav:fr:eph").unwrap(), target);

		assert_eq!(TargetItem::from_str("nav:fr:eph, ion").unwrap(), 
			TargetItem::NavFrameItem(
				vec![FrameClass::Ephemeris, FrameClass::IonosphericModel]));

		assert_eq!(TargetItem::from_str("sv:g08,g09,R03").unwrap(), 
			TargetItem::SvItem(
				vec![Sv::from_str("G08").unwrap(),
				Sv::from_str("G09").unwrap(),
				Sv::from_str("R03").unwrap()]));

        assert_eq!(TargetItem::from_str("gnss:GPS , BDS").unwrap(),
            TargetItem::ConstellationItem(vec![Constellation::GPS, Constellation::BeiDou]));

        let dt = Duration::from_str("1 d").unwrap();
        let target: TargetItem = dt.into();
        assert_eq!(target, TargetItem::DurationItem(dt));
        assert_eq!(TargetItem::from_str("dt: 1 d").unwrap(), target);
    }
}
