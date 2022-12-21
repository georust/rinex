mod sampling;
pub use sampling::Decimation;

mod mask;
pub use mask::{Mask, MaskFilter, MaskOperand};

//mod processing;
//pub use processing::{Processing, AverageType};

use thiserror::Error;
use crate::navigation::{MsgType, FrameClass};
use crate::{Epoch, EpochFlag, Sv, Constellation, Observable};

/// Target Item represents items that filter operations
/// or algorithms may target
#[derive(Clone, Debug, PartialEq)]
pub enum TargetItem {
    /// RINEX data on Epoch
    EpochItem(Epoch),
    /// Filter Observation RINEX on Epoch Flag
    EpochFlagItem(EpochFlag),
    /// Filter Navigation RINEX on elevation angle 
    ElevationItem(f64),
    /// Filter RINEX data on list of vehicle
    SvItem(Vec<Sv>),
    /// Filter RINEX data on list of constellation
    ConstellationItem(Vec<Constellation>),
    /// Filter Observation RINEX on list of observables 
    ObservableItem(Vec<Observable>),
    /// Filter Navigation RINEX on list of Orbit item 
    OrbitItem(Vec<String>),
    /// Filter Navigation RINEX on Message type 
    NavMsgItem(Vec<MsgType>),
    /// Filter Navigation RINEX on Frame type 
    NavFrameItem(Vec<FrameClass>),
}

impl std::str::FromStr for TargetItem {
    type Err = AlgorithmError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        if let Ok(epoch) = Epoch::from_str(c) {
            Ok(Self::EpochItem(epoch))
		} else if let Ok(flag) = EpochFlag::from_str(c) {
            Ok(Self::EpochFlagItem(flag))
        } else if let Ok(f) = f64::from_str(c) {
			Ok(Self::ElevationItem(f))
		} else {
			/*
			 * others support a vector of items
			 */
			 let items: Vec<&str> = c.split(",").collect();
			 let mut svs: Vec<Sv> = Vec::with_capacity(items.len());
			 let mut consts: Vec<Constellation> = Vec::with_capacity(items.len());
			 let mut obss: Vec<Observable> = Vec::with_capacity(items.len());
			 let mut orbs: Vec<String> = Vec::with_capacity(items.len());
			 let mut fr: Vec<FrameClass> = Vec::with_capacity(items.len());
			 let mut msg: Vec<MsgType> = Vec::with_capacity(items.len());
			 for item in items {
				if let Ok(sv) = Sv::from_str(item.trim()) {
					svs.push(sv);
				} else if let Ok(c) = Constellation::from_str(item.trim()) {
					consts.push(c);
				} else if let Ok(m) = MsgType::from_str(item.trim()) {
					msg.push(m);
				} else if let Ok(f) = FrameClass::from_str(item.trim()) {
					fr.push(f);
				} else if let Ok(ob) = Observable::from_str(item.trim()) {
					obss.push(ob);
				} else {
					orbs.push(item.trim().to_string());
				}
			}
			if svs.len() > 0 {
				Ok(Self::SvItem(svs))
			} else if consts.len() > 0 {
				Ok(Self::ConstellationItem(consts))
			} else if obss.len() > 0 {
				Ok(Self::ObservableItem(obss))
			} else if fr.len() > 0 {
				Ok(Self::NavFrameItem(fr))
			} else if msg.len() > 0 {
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

#[derive(Clone, Debug, Error, PartialEq)]
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
    #[error("invalid nav frame type")]
    InvalidNavFrame,
    #[error("invalid nav message type")]
    InvalidNavMsg,
    #[error("invalid nav filter")]
    InvalidNavFilter,
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
        assert_eq!(TargetItem::from_str("0"), Ok(target));
        
        let obs = Observable::default();
        let target: TargetItem = obs.clone().into();
        assert_eq!(target, TargetItem::ObservableItem(vec![obs.clone()]));
        assert_eq!(TargetItem::from_str("L1C"), Ok(target));
        
        let msg = MsgType::LNAV;
        let target: TargetItem = msg.into();
        assert_eq!(target, TargetItem::NavMsgItem(vec![msg]));
        assert_eq!(TargetItem::from_str("LNAV"), Ok(target));
        
        let fr = FrameClass::Ephemeris;
        let target: TargetItem = fr.into();
        assert_eq!(target, TargetItem::NavFrameItem(vec![fr]));
        assert_eq!(TargetItem::from_str("EPH"), Ok(target));

		assert_eq!(TargetItem::from_str("eph, ion"), 
			Ok(TargetItem::NavFrameItem(
				vec![FrameClass::Ephemeris, FrameClass::IonosphericModel])));

		assert_eq!(TargetItem::from_str("g08,g09,R03"), 
			Ok(TargetItem::SvItem(
				vec![Sv::from_str("G08").unwrap(),
				Sv::from_str("G09").unwrap(),
				Sv::from_str("R03").unwrap()])));

        assert_eq!(TargetItem::from_str("GPS , BDS"),
            Ok(TargetItem::ConstellationItem(vec![Constellation::GPS, Constellation::BeiDou])));
    }
}
