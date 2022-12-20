use thiserror::Error;
use std::str::FromStr;
use super::{AlgorithmError, TargetItem};
use crate::{Epoch, EpochFlag, Sv, Constellation};

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
    type Err = AlgorithmError;
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
            Err(AlgorithmError::UnknownOperand)
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

/// FilterOp is an effecient structure to describe high level
/// focus / filtering operations.
/// See [Filter::apply] definition, for example of use.
/// ```
/// use rinex::prelude::*;
/// use rinex::processing::*;
///
/// // after "epoch" condition
/// let after_filter = FilterOp::from_str("> 2022-01-01 10:00:00UTC")
///     .unwrap();
/// // any valid Epoch description will work 
/// let after_filter = FilterOp::from_str("> JD 2960")
///     .unwrap();
///	
///	// complex operation: 
///	// to describe several conditions at once, delimit them with ";"
///	// Theoretically, we support an infinite number of conditions.
///	// For example: here we describe a precise epoch interval
///	let interval = FilterOp::from_str(">  2022-01-01 10:00:00UTC ; <= 2022-01-01 10:00:10UTC")
///	    .unwrap();
/// 
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FilterOp {
    pub operand: FilterOperand,
    pub targets: Vec<TargetItem>,
}

impl std::str::FromStr for FilterOp {
    type Err = AlgorithmError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = content.trim();
        if content.len() < 3 {
            return Err(AlgorithmError::MalformedDescriptor);
        }
        if let Ok(operand) = FilterOperand::from_str(content) {
            let offset = operand.formatted_len();
            let items = &content[offset..];
            let items: Vec<&str> = items.split(";")
                .collect();
            let mut targets: Vec<TargetItem> = Vec::with_capacity(items.len()); 
            for item in items {
                targets.push(
                    TargetItem::from_str(&content[offset..])?
                );
            }
            Ok(Self {
                operand,
                targets,
            })
        } else {
            Err(AlgorithmError::UnknownOperand)
        }
    }
}

pub trait Filter {
	/// Applies given filter to self.
	/// ```
	/// use rinex::prelude::*;
	/// use rinex::processing::*;
	/// // parse a RINEX file
	/// let rinex = Rinex::from_file("../test_resources/OBS/V3/")
	///		.unwrap();
    ///
    /// // design a filter
	/// let sv_filter: FilterOp::from_str("= GPS")
	///		.unwrap();
	/// let rinex = rinex.filter(sv_filter);
    ///
	/// // design a filter: case insensitive
	/// let sv_filter: FilterOp::from_str("= g08,g09")
	///		.unwrap();
	/// let rinex = rinex.filter(sv_filter);
    ///
	/// // whitespace is not mandatory,
	/// let phase_filter = FilterOp::from_str("=L1C,l2l,L2W")
	///		.unwrap();
	/// let rinex = rinex.filter(sv_filter);
    ///
    /// // if the descriptor is a float number,
    /// // we consider it to be an elevation mask as of today.
    /// // Maybe in the future, we'll have options to describe
    /// // other types of fields or physics
    /// let elev_mask = FilterOp::from_str(">= 33.0") // elev mask in °
    ///     .unwrap();
    /// // Applying a mask like this can only work 
    /// // on Navigation Data: this is just an example.
    /// let rinex = rinex.filter(elev_mask);
	/// ```
    fn apply(&self, filter: FilterOp) -> Self;
	/// Mutable implementation, see [Filter::apply]
    fn apply_mut(&mut self, filter: FilterOp);
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
        let mask = FilterOp::from_str("> 2020-01-14T00:31:55 UTC");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::StrictlyAbove,
                targets: vec![
                    TargetItem::EpochItem(Epoch::from_str("2020-01-14T00:31:55 UTC").unwrap())
                ],
            }));
        let mask = FilterOp::from_str("> JD 2452312.500372511 TAI");
        assert!(mask.is_ok());
    }
    #[test]
    fn test_elev_mask() {
        let mask = FilterOp::from_str("< 40.0");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::StrictlyBelow,
                targets: vec![TargetItem::ElevationItem(40.0_f64)],
            }));
        let m2 = FilterOp::from_str("<40.0");
        assert_eq!(mask, m2);
        let m2 = FilterOp::from_str("  < 40.0  ");
        assert_eq!(mask, m2);

        let mask = FilterOp::from_str(">= 10.0");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::Above,
                targets: vec![TargetItem::ElevationItem(10.0_f64)],
            }));
        let m2 = FilterOp::from_str(">=10.0");
        assert_eq!(mask, m2);
    }
    /*
    #[test]
    fn test_constell_mask() {
        let mask = FilterOp::from_str("= c: GPS");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::Equal,
                targets: TargetItem::ConstellationFilter(vec![Constellation::GPS]),
            }));
        let m2 = FilterOp::from_str("=c: GPS");
        assert_eq!(mask, m2);

        let mask = FilterOp::from_str("= c: GPS,GAL,GLO");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::Equal,
                targets: TargetItem::ConstellationFilter(vec![Constellation::GPS, Constellation::Galileo, Constellation::Glonass]),
            }));
        let m2 = FilterOp::from_str("=c: GPS,GAL,GLO");
        assert_eq!(mask, m2);
        
        let mask = FilterOp::from_str("!= c: BDS");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::NotEqual,
                targets: TargetItem::ConstellationFilter(vec![Constellation::BeiDou]),
            }));
        let m2 = FilterOp::from_str("!=c:BDS");
        assert_eq!(mask, m2);
    }
    #[test]
    fn test_sv_mask() {
        let mask = FilterOp::from_str("= sv: G08,  G09, R03");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::Equal,
                targets: TargetItem::SvItem(vec![
                    Sv::from_str("G08").unwrap(),
                    Sv::from_str("G09").unwrap(),
                    Sv::from_str("R03").unwrap(),
                ]),
            }));
        let m2 = FilterOp::from_str("= sv: G08,G09,R03");
        assert_eq!(mask, m2);
        
        let mask = FilterOp::from_str("!= sv: G31");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::NotEqual,
                targets: TargetItem::SvItem(vec![
                    Sv::from_str("G31").unwrap(),
                ]),
            }));
        let m2 = FilterOp::from_str("!=sv:G31");
        assert_eq!(mask, m2);
    }
    #[test]
    fn test_obs_mask() {
        let mask = FilterOp::from_str("= obs: ph,ssi,pr");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::Equal,
                targets: TargetItem::ObservableItem(
                    vec![String::from("ph"), String::from("ssi"), String::from("pr")])
            }));
    }
    #[test]
    fn test_orb_mask() {
        let mask = FilterOp::from_str("= iode");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::Equal,
                targets: TargetItem::OrbitItem(vec![String::from("iode")])
            }));
    }
    #[test]
    fn test_nav_mask() {
        let mask = FilterOp::from_str("= eph");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::Equal,
                targets: vec![TargetItem::NavFrameItem(FrameClass::Ephemeris)],
            }));
        let mask = FilterOp::from_str("= lnav");
        assert_eq!(
            mask,
            Ok(FilterOp {
                operand: FilterOperand::Equal,
                targets: vec![TargetItem::NavMsgItem(MsgType::LNAV)],
            }));
    }
    */
}
