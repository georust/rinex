mod sampling;
pub use sampling::Decimation;

mod filter;
pub use filter::{Filter, MaskFilter, FilterOperand};

//mod processing;
//pub use processing::{Processing, AverageType};

use thiserror::Error;
use std::str::FromStr;
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
    SvItem(Sv),
    /// Filter RINEX data on list of constellation
    ConstellationItem(Constellation),
    /// Filter Observation RINEX on list of observables 
    ObservableItem(Observable),
    /// Filter Navigation RINEX on list of Orbit item 
    OrbitItem(String),
    /// Filter Navigation RINEX on Message type 
    NavMsgItem(MsgType),
    /// Filter Navigation RINEX on Frame type 
    NavFrameItem(FrameClass),
}

impl std::str::FromStr for TargetItem {
    type Err = AlgorithmError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        if let Ok(epoch) = Epoch::from_str(c) {
            Ok(Self::EpochItem(epoch))
		} else if let Ok(sv) = Sv::from_str(c) {
			Ok(Self::SvItem(sv))
		} else if let Ok(c) = Constellation::from_str(c) {
			Ok(Self::ConstellationItem(c))
		} else if let Ok(msg) = MsgType::from_str(c) {
			Ok(Self::NavMsgItem(msg))
		} else if let Ok(fr) = FrameClass::from_str(c) {
			Ok(Self::NavFrameItem(fr))
		} else if let Ok(obs) = Observable::from_str(c) {
            Ok(Self::ObservableItem(obs))
        } else {
            Err(AlgorithmError::UnrecognizedTarget)
        }
    }
}

impl From<Epoch> for TargetItem {
    fn from(e: Epoch) -> Self {
        Self::EpochItem(e)
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
