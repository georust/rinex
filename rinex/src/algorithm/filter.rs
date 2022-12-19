use thiserror::Error;
use std::str::FromStr;
use crate::{Epoch, EpochFlag, Sv, Constellation};
use crate::meteo::Observable;
use crate::navigation::MsgType;
use crate::navigation::FrameClass;

#[derive(Debug, Clone, PartialEq)]
pub enum FilterOperand {
    Above,
    StrictlyAbove,
    Below,
    StrictlyBelow,
    Equal,
    NotEqual,
}

impl std::str::FromStr for FilterOperand {
    type Err = FilterParsingError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        if c.starts_with(">=") {
            Ok(Self::Above)
        } else if c.starts_with(">") {
            Ok(Self::StrictlyAbove)
        } else if c.starts_with("<=") {
            Ok(Self::Below)
        } else if c.starts_with("<") {
            Ok(Self::StrictlyBelow)
        } else if c.starts_with("=") {
            Ok(Self::Equal)
        } else if c.starts_with("!=") {
            Ok(Self::NotEqual)
        } else {
            Err(FilterParsingError::UnknownOperand)
        }
    }
}

impl FilterOperand {
    pub(crate) const fn formatted_len(&self) -> usize {
        match &self {
            Self::NotEqual | Self::Below | Self::Above => 2,
            Self::Equal | Self::StrictlyBelow | Self::StrictlyAbove => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaskFilter<I> {
    pub operand: FilterOperand,
    pub item: I,
}

impl std::str::FromStr for MaskFilter<FilterItem> {
    type Err = FilterParsingError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = content.trim();
        if content.len() < 3 {
            return Err(FilterParsingError::MalformedDescriptor);
        }
        if let Ok(operand) = FilterOperand::from_str(content) {
            let offset = operand.formatted_len();
            let item = FilterItem::from_str(&content[offset..])?;
            Ok(Self { operand, item })
        } else {
            Err(FilterParsingError::UnknownOperand)
        }
    }
}

/// Filter item is what a `Filter` actually targets
#[derive(Clone, Debug, PartialEq)]
pub enum FilterItem {
    /// Filter RINEX data on Epoch
    EpochFilter(Epoch),
    /// Filter Observation RINEX on Epoch Flag
    EpochFlagFilter(EpochFlag),
    /// Filter Navigation RINEX on elevation angle 
    ElevationFilter(f64),
    /// Filter RINEX data on list of vehicle
    SvFilter(Vec<Sv>),
    /// Filter RINEX data on list of constellation
    ConstellationFilter(Vec<Constellation>),
    /// Filter Observation RINEX on list of observables 
    ObservableFilter(Vec<String>),
    /// Filter Navigation RINEX on list of Orbit item 
    OrbitFilter(Vec<String>),
    /// Filter Navigation RINEX on Message type 
    NavMsgFilter(Vec<MsgType>),
    /// Filter Navigation RINEX on Frame type 
    NavFrameFilter(Vec<FrameClass>),
}

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

impl std::str::FromStr for FilterItem {
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
    fn apply(&self, mask: MaskFilter<FilterItem>) -> Self;
    fn apply_mut(&mut self, mask: MaskFilter<FilterItem>);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_mask_operand() {
        let operand = FilterOperand::from_str(">");
        assert_eq!(operand, Ok(FilterOperand::StrictlyAbove));

        let operand = FilterOperand::from_str(">=");
        assert_eq!(operand, Ok(FilterOperand::Above));

        let operand = FilterOperand::from_str("<");
        assert_eq!(operand, Ok(FilterOperand::StrictlyBelow));

        let operand = FilterOperand::from_str("<=");
        assert_eq!(operand, Ok(FilterOperand::Below));

        let operand = FilterOperand::from_str("<= 123");
        assert_eq!(operand, Ok(FilterOperand::Below));

        let operand = FilterOperand::from_str(">123.0");
        assert_eq!(operand, Ok(FilterOperand::StrictlyAbove));

        let operand = FilterOperand::from_str(">abcdefg");
        assert_eq!(operand, Ok(FilterOperand::StrictlyAbove));

        let operand = FilterOperand::from_str("!>");
        assert!(operand.is_err());
    }
    #[test]
    fn test_epoch_mask() {
        let mask = MaskFilter::from_str("> 2020-01-14T00:31:55 UTC");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::StrictlyAbove,
                item: FilterItem::EpochFilter(Epoch::from_str("2020-01-14T00:31:55 UTC").unwrap()),
            }));
        let mask = MaskFilter::from_str("> JD 2452312.500372511 TAI");
        assert!(mask.is_ok());
    }
    #[test]
    fn test_elev_mask() {
        let mask = MaskFilter::from_str("< e: 40.0");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::StrictlyBelow,
                item: FilterItem::ElevationFilter(40.0_f64),
            }));
        let m2 = MaskFilter::from_str("<e: 40.0");
        assert_eq!(mask, m2);

        let mask = MaskFilter::from_str(">= e: 10.0");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Above,
                item: FilterItem::ElevationFilter(10.0_f64),
            }));
        let m2 = MaskFilter::from_str(">=e: 10.0");
        assert_eq!(mask, m2);
    }
    #[test]
    fn test_constell_mask() {
        let mask = MaskFilter::from_str("= c: GPS");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Equal,
                item: FilterItem::ConstellationFilter(vec![Constellation::GPS]),
            }));
        let m2 = MaskFilter::from_str("=c: GPS");
        assert_eq!(mask, m2);

        let mask = MaskFilter::from_str("= c: GPS,GAL,GLO");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Equal,
                item: FilterItem::ConstellationFilter(vec![Constellation::GPS, Constellation::Galileo, Constellation::Glonass]),
            }));
        let m2 = MaskFilter::from_str("=c: GPS,GAL,GLO");
        assert_eq!(mask, m2);
        
        let mask = MaskFilter::from_str("!= c: BDS");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::NotEqual,
                item: FilterItem::ConstellationFilter(vec![Constellation::BeiDou]),
            }));
        let m2 = MaskFilter::from_str("!=c:BDS");
        assert_eq!(mask, m2);
    }
    #[test]
    fn test_sv_mask() {
        let mask = MaskFilter::from_str("= sv: G08,  G09, R03");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Equal,
                item: FilterItem::SvFilter(vec![
                    Sv::from_str("G08").unwrap(),
                    Sv::from_str("G09").unwrap(),
                    Sv::from_str("R03").unwrap(),
                ]),
            }));
        let m2 = MaskFilter::from_str("= sv: G08,G09,R03");
        assert_eq!(mask, m2);
        
        let mask = MaskFilter::from_str("!= sv: G31");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::NotEqual,
                item: FilterItem::SvFilter(vec![
                    Sv::from_str("G31").unwrap(),
                ]),
            }));
        let m2 = MaskFilter::from_str("!=sv:G31");
        assert_eq!(mask, m2);
    }
    #[test]
    fn test_obs_mask() {
        let mask = MaskFilter::from_str("= obs: ph,ssi,pr");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Equal,
                item: FilterItem::ObservableFilter(
                    vec![String::from("ph"), String::from("ssi"), String::from("pr")])
            }));
    }
    #[test]
    fn test_orb_mask() {
        let mask = MaskFilter::from_str("= orb: iode");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Equal,
                item: FilterItem::OrbitFilter(vec![String::from("iode")])
            }));
    }
    #[test]
    fn test_nav_mask() {
        let mask = MaskFilter::from_str("= nav:fr:eph");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Equal,
                item: FilterItem::NavFrameFilter(vec![FrameClass::Ephemeris])
            }));
        let mask = MaskFilter::from_str("= nav:msg:lnav");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Equal,
                item: FilterItem::NavMsgFilter(vec![MsgType::LNAV])
            }));
    }
}
