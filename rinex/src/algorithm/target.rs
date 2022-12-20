use thiserror::Error;
use std::str::FromStr;
use crate::{Epoch, EpochFlag, Sv, Constellation};
use crate::meteo::Observable;
use crate::navigation::MsgType;
use crate::navigation::FrameClass;


#[derive(Clone, Debug, Error, PartialEq)]
pub enum FilterParsingError {
    #[error("unrecognized operand")]
    UnknownOperand,
    #[error("unrecognized item")]
    UnrecognizedItem,
    #[error("malformed description")]
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

impl std::str::FromStr for TargetItem {
    type Err = FilterParsingError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        if let Ok(epoch) = Epoch::from_str(c) {
            Ok(Self::EpochFilter(epoch))
        } else {
            if !c.contains(":") {
                Err(FilterParsingError::MalformedDescriptor)
            } else {
                let items: Vec<&str> = c.split(":")
                    .collect();
                if items.len() < 2 {
                    Err(FilterParsingError::MalformedDescriptor)
                } else {
                    if items[0].trim().eq("e") {
                        let value = f64::from_str(items[1].trim())?;
                        Ok(Self::ElevationFilter(value))
                    } else if items[0].trim().eq("f") {
                        let value = EpochFlag::from_str(items[1].trim())?;
                        Ok(Self::EpochFlagFilter(value))
                    } else if items[0].trim().eq("c") {
                        let desc: Vec<&str> = items[1].split(",").collect();
                        let mut constells: Vec<Constellation> = Vec::with_capacity(desc.len()); 
                        for item in desc {
                            let c = Constellation::from_str(item.trim())?;
                            constells.push(c);
                        }
                        Ok(Self::ConstellationFilter(constells))
                    } else if items[0].trim().eq("sv") {
                        let desc: Vec<&str> = items[1].split(",").collect();
                        let mut svs: Vec<Sv> = Vec::with_capacity(desc.len()); 
                        for item in desc {
                            let sv = Sv::from_str(item.trim())?;
                            svs.push(sv);
                        }
                        Ok(Self::SvFilter(svs))
                    } else if items[0].trim().eq("obs") {
                        let desc: Vec<String> = items[1].split(",")
                            .filter_map(|s| {
                                if Observable::from_str(s.trim()).is_ok() {
                                    Some(s.to_string())
                                } else if s.trim().eq("ph") { 
                                    Some(s.trim().to_string())
                                } else if s.trim().eq("ssi") {
                                    Some(s.to_string())
                                } else if s.trim().eq("pr") {
                                    Some(s.to_string())
                                } else if s.trim().eq("dop") {
                                    Some(s.to_string())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        Ok(Self::ObservableFilter(desc))
                    } else if items[0].trim().eq("orb") {
                        let desc: Vec<String> = items[1].split(",")
                            .map(|s| s.trim().to_string())
                            .collect();
                        Ok(Self::OrbitFilter(desc))
                    } else if items[0].trim().eq("nav") {
                        if items.len() < 3 {
                            return Err(FilterParsingError::MalformedDescriptor);
                        }
                        if items[1].eq("fr") {
                            let items: Vec<&str> = items[2].split(",")
                                .collect();
                            let mut filter: Vec<FrameClass> = Vec::with_capacity(items.len());
                            for item in items {
                                if let Ok(fr) = FrameClass::from_str(item.trim()) {
                                    filter.push(fr)
                                } else {
                                    return Err(FilterParsingError::InvalidNavFrame);
                                }
                            }
                            Ok(Self::NavFrameFilter(filter))
                        } else if items[1].eq("msg") {
                            let items: Vec<&str> = items[2].split(",")
                                .collect();
                            let mut filter: Vec<MsgType> = Vec::with_capacity(items.len());
                            for item in items {
                                if let Ok(msg) = MsgType::from_str(item.trim()) {
                                    filter.push(msg)
                                } else {
                                    return Err(FilterParsingError::InvalidNavMsg);
                                }
                            }
                            Ok(Self::NavMsgFilter(filter))
                        } else {
                            return Err(FilterParsingError::InvalidNavFilter);
                        }
                    } else {
                        Err(FilterParsingError::UnrecognizedItem)
                    }
                }
            }
        } 
    }
}

pub trait Filter {
	/// Applies given filter to self.
	/// ```
	/// use rinex::prelude::*;
	/// use rinex::processing::*;
	/// // parse a RINEX file
	/// let mut rinex = Rinex::from_file("../test_resources/OBS/V3/")
	///		.unwrap();
	/// // design a filter
	/// let sv_filt: MaskFilter::from_str("= sv: G08,G09")
	///		.unwrap();
	/// rinex.filter_mut(sv_filt);
	/// // design a filter
	/// let phase_filt = MaskFilter::from_str("= obs: ph")
	///		.unwrap();
	/// rinex.filter_mut(phase_filt);
	/// // apply a time window
	/// let start = MaskFilter::from_str("> 2022
	/// ```
    fn apply(&self, mask: MaskFilter<TargetItem>) -> Self;
	/// Mutable implementation, see [Filter::apply]
    fn apply_mut(&mut self, mask: MaskFilter<TargetItem>);
}
