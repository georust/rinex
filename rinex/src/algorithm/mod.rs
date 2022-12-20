mod sampling;
pub use sampling::Decimation;

mod filter;
pub use filter::{Filter, MaskFilter, FilterOperand, FilterParsingError};
pub use TargetItem;

//mod processing;
//pub use processing::{Processing, AverageType};

use crate::meteo::Observable;
use crate::navigation::MsgType;
use crate::navigation::FrameClass;

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
    ObservableItem(String),
    /// Filter Navigation RINEX on list of Orbit item 
    OrbitItem(String),
    /// Filter Navigation RINEX on Message type 
    NavMsgItem(MsgType),
    /// Filter Navigation RINEX on Frame type 
    NavFrameItem(FrameClass),
}

impl std::str::FromStr for TargetItem {
    type Err = FilterParsingError;
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
		} else if let Ok(fr) = FrameClas::from_str(c) {
			Ok(Self::NavFrameItem(fr)))
		} else if let Ok(obs) = Observable::from_str(c) { 

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
