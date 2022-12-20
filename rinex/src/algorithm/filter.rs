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

/// MaskFilter is an effecient structure to describe high level
/// operations, to focus on data or subset of interest
/// ```
/// use rinex::prelude::*;
/// use rinex::processing::*;
///
/// // after "epoch" condition
/// let after_mask: MaskFilter::from_str("> e: 2022-01-01 10:00:00UTC")
///		.unwrap();
/// // any valid Epoch description is available
/// let after_mask: MaskFilter::from_str("> e: JD 2960") 
///		.unwrap();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MaskFilter<T> {
    pub operand: FilterOperand,
    pub targets: Vec<T>,
}

impl std::str::FromStr for MaskFilter<TargetItem> {
    type Err = AlgorithmError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = content.trim();
        if content.len() < 3 {
            return Err(AlgorithmError::MalformedDescriptor);
        }
        if let Ok(operand) = FilterOperand::from_str(content) {
            let offset = operand.formatted_len();
            let item = TargetItem::from_str(&content[offset..])?;
            Ok(Self { operand, item })
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
	/// let sv_filter: MaskFilter::from_str("= GPS")
	///		.unwrap();
	/// let rinex = rinex.filter(sv_filter);
    ///
	/// // design a filter: case insensitive
	/// let sv_filter: MaskFilter::from_str("= g08,g09")
	///		.unwrap();
	/// let rinex = rinex.filter(sv_filter);
    ///
	/// // whitespace is not mandatory,
	/// let phase_filter = MaskFilter::from_str("=L1C,l2l,L2W")
	///		.unwrap();
	/// let rinex = rinex.filter(sv_filter);
    ///
	/// // apply a time window
	/// let start = MaskFilter::from_str("> 2022
	/// ```
    fn apply(&self, mask: MaskFilter<TargetItem>) -> Self;
	/// Mutable implementation, see [Filter::apply]
    fn apply_mut(&mut self, mask: MaskFilter<TargetItem>);
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
                item: TargetItem::EpochFilter(Epoch::from_str("2020-01-14T00:31:55 UTC").unwrap()),
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
                item: TargetItem::ElevationFilter(40.0_f64),
            }));
        let m2 = MaskFilter::from_str("<e: 40.0");
        assert_eq!(mask, m2);

        let mask = MaskFilter::from_str(">= e: 10.0");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Above,
                item: TargetItem::ElevationFilter(10.0_f64),
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
                item: TargetItem::ConstellationFilter(vec![Constellation::GPS]),
            }));
        let m2 = MaskFilter::from_str("=c: GPS");
        assert_eq!(mask, m2);

        let mask = MaskFilter::from_str("= c: GPS,GAL,GLO");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Equal,
                item: TargetItem::ConstellationFilter(vec![Constellation::GPS, Constellation::Galileo, Constellation::Glonass]),
            }));
        let m2 = MaskFilter::from_str("=c: GPS,GAL,GLO");
        assert_eq!(mask, m2);
        
        let mask = MaskFilter::from_str("!= c: BDS");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::NotEqual,
                item: TargetItem::ConstellationFilter(vec![Constellation::BeiDou]),
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
                item: TargetItem::SvFilter(vec![
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
                item: TargetItem::SvFilter(vec![
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
                item: TargetItem::ObservableFilter(
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
                item: TargetItem::OrbitFilter(vec![String::from("iode")])
            }));
    }
    #[test]
    fn test_nav_mask() {
        let mask = MaskFilter::from_str("= nav:fr:eph");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Equal,
                item: TargetItem::NavFrameFilter(vec![FrameClass::Ephemeris])
            }));
        let mask = MaskFilter::from_str("= nav:msg:lnav");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: FilterOperand::Equal,
                item: TargetItem::NavMsgFilter(vec![MsgType::LNAV])
            }));
    }
}
