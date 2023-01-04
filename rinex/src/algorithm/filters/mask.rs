use thiserror::Error;
use crate::processing::TargetItem;

#[derive(Error, Debug)]
pub enum Error {
	#[error("invalid mask target")]
	TargetError(#[from] crate::algorithm::target::Error),
	#[error("invalid mask operand")]
	InvalidOperand,
	#[error("invalid mask description")]
	InvalidDescriptor,
}

pub trait Mask {
    fn mask(&self, mask: MaskFilter) -> Self;
    fn mask_mut(&mut self, mask: MaskFilter);
}

/// MaskOperand describe how to apply a mask
/// in related filter operation
#[derive(Debug, Clone, PartialEq)]
pub enum MaskOperand {
	/// Greater than
    GreaterThan,
	/// Greater Equals
    GreaterEquals,
	/// Lower than
    LowerThan,
	/// Lower Equals
	LowerEquals,
	/// Equals
    Equals,
	/// Not Equals
    NotEquals,
}

impl std::str::FromStr for MaskOperand {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        if c.starts_with("geq") {
            Ok(Self::GreaterEquals)
        } else if c.starts_with("gt") {
            Ok(Self::GreaterThan)
        } else if c.starts_with("leq") {
            Ok(Self::LowerEquals)
        } else if c.starts_with("lt") {
            Ok(Self::LowerThan)
        } else if c.starts_with("eq") {
            Ok(Self::Equals)
        } else if c.starts_with("neq") {
            Ok(Self::NotEquals)
        } else {
            Err(Error::InvalidOperand)
        }
    }
}

impl MaskOperand {
    pub(crate) const fn formatted_len(&self) -> usize {
        match &self {
			Self::Equals | Self::GreaterThan | Self::LowerThan => 2,
			Self::NotEquals | Self::LowerEquals | Self::GreaterEquals => 3,
        }
    }
}

impl std::ops::Not for MaskOperand {
    type Output = MaskOperand;
    fn not(self) -> Self {
        match self {
            Self::Equals => Self::NotEquals,
            Self::NotEquals => Self::Equals,
            Self::GreaterEquals => Self::LowerThan,
            Self::GreaterThan => Self::LowerEquals,
            Self::LowerThan => Self::GreaterEquals,
            Self::LowerEquals => Self::GreaterThan,
        }
    }
}

/// A Mask is an effecient way to describe a filter operation
/// to apply on a RINEX record.
/// See [Mask::apply] definition for compelling examples.
/// ```
/// use rinex::prelude::*;
/// use rinex::processing::*;
/// // after "epoch" condition
/// let after = Mask::from_str("gt:2022-01-01 10:00:00UTC")
///     .unwrap();
/// let before = Mask::from_str("leq:2022-01-01 10:00:00UTC")
///     .unwrap();
/// assert_eq!(before, !after); // logical not() is supported for all mask operands
///
/// // the payload can be any valid Epoch Description,
/// // refer to [Hifitime::Epoch]
/// let equals = Mask::from_str("eq:JD 2960")
///     .unwrap();
/// assert_eq!(Mask::from_str("neq:JD 2960").unwrap(), !equals);
///
/// // mask can apply to a lot of different data subsets,
/// // refer to the [TargetItem] definition,
///
/// // Greater than ">" and lower than "<"
/// // truly apply to Epochs and Durations only,
/// // whereas Equality masks ("=", "!=") apply to any known item.
/// // One exception exist for "Sv" items, for example with this:
/// let greater_than = Mask::from_str("gt: G08,R03")
///     .unwrap();
/// // will retain PRN >= 08 for GPS and PRN >= 3 for Glonass
///
/// // Logical not() is supported for a mask filter:
/// let lower_than = Mask::from_str("leq: G08,R03")
///     .unwrap();
/// assert_eq!(lower_than, !greater_than);
///
/// // Logical Or() is supported, and allows putting toghether
/// // an array of values. It does not apply to Epoch, Duration Items for example..
/// let mut observable_mask = Mask::from_str("eq: L1C")
///     .unwrap();
/// observable_mask |= Mask::from_str("eq: L1P").unwrap();
/// assert_eq!(observable_mask, Mask::from_str("eq: L1C,L1P").unwrap());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MaskFilter {
    pub operand: MaskOperand,
    pub item: TargetItem,
}

impl std::ops::Not for MaskFilter {
    type Output = MaskFilter;
    fn not(self) -> Self {
        Self {
            operand: !self.operand,
            item: self.item,
        }
    }
}

impl std::ops::BitOr for MaskFilter {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        if self.operand == rhs.operand {
            Self {
                operand: self.operand,
                item: self.item | rhs.item,
            }
        } else {
            // not permitted
            self.clone()
        }
    }
}

impl std::ops::BitOrAssign for MaskFilter {
    fn bitor_assign(&mut self, rhs: Self) {
        self.item = self.item.clone() | rhs.item;
    }
}

impl std::str::FromStr for MaskFilter {
	type Err = Error;
   	fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.split(":")
			.collect();
        if items.len() < 2 {
            return Err(Error::InvalidDescriptor);
        }
        if let Ok(operand) = MaskOperand::from_str(items[0].trim()) {
            let offset = operand.formatted_len();
            Ok(Self {
                operand,
                item: TargetItem::from_str(&content[offset+1..].trim())?, // +1: ":"
            })
        } else {
            Err(Error::InvalidOperand)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::navigation::{FrameClass, MsgType};
    use crate::prelude::*;
    use std::str::FromStr;
    #[test]
    fn mask_operand() {
        let operand = MaskOperand::from_str("geq").unwrap();
        assert_eq!(operand, MaskOperand::GreaterEquals);
        assert_eq!(!operand, MaskOperand::from_str("lt").unwrap());

        let operand = MaskOperand::from_str("gt").unwrap();
        assert_eq!(operand, MaskOperand::GreaterThan);
        assert_eq!(!operand, MaskOperand::from_str("leq").unwrap());

        let operand = MaskOperand::from_str("lt").unwrap();
        assert_eq!(operand, MaskOperand::LowerThan);
        assert_eq!(!operand, MaskOperand::from_str("geq").unwrap());

        let operand = MaskOperand::from_str("leq").unwrap();
        assert_eq!(operand, MaskOperand::LowerEquals);
        assert_eq!(!operand, MaskOperand::from_str("gt").unwrap());

        let operand = MaskOperand::from_str("geq: 123").unwrap();
        assert_eq!(operand, MaskOperand::GreaterEquals);
        assert_eq!(!operand, MaskOperand::LowerThan);

        let operand = MaskOperand::from_str("geq:123.0").unwrap();
        assert_eq!(operand, MaskOperand::GreaterEquals);
        assert_eq!(!operand, MaskOperand::from_str("lt").unwrap());

        let operand = MaskOperand::from_str("gt:abcdefg").unwrap();
        assert_eq!(operand, MaskOperand::GreaterThan);
        assert_eq!(!operand, MaskOperand::from_str("leq").unwrap());

        let operand = MaskOperand::from_str("!>");
        assert!(operand.is_err());
    }
    #[test]
    fn mask_epoch() {
        let mask = MaskFilter::from_str("gt:2020-01-14T00:31:55 UTC").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::GreaterThan,
                item: TargetItem::EpochItem(Epoch::from_str("2020-01-14T00:31:55 UTC").unwrap()),
            }
        );
        let mask = MaskFilter::from_str("gt:JD 2452312.500372511 TAI");
        assert!(mask.is_ok());
    }
    #[test]
    fn mask_elev() {
        let mask = MaskFilter::from_str("lt:elev: 40.0").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::LowerThan,
                item: TargetItem::ElevationItem(40.0_f64),
            }
        );
        let mask = MaskFilter::from_str("geq:elev:10.0").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::GreaterEquals,
                item: TargetItem::ElevationItem(10.0_f64),
            }
        );
        let m2 = MaskFilter::from_str("lt:elev:10.0").unwrap();
        assert_eq!(!mask, m2);
    }
    #[test]
    fn mask_gnss() {
        let mask = MaskFilter::from_str("eq: GPS").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::ConstellationItem(vec![Constellation::GPS]),
            }
        );
        let m2 = MaskFilter::from_str("eq:GPS").unwrap();
        assert_eq!(mask, m2);

        let mask = MaskFilter::from_str("eq: GPS,GAL,GLO").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::ConstellationItem(vec![
                    Constellation::GPS,
                    Constellation::Galileo,
                    Constellation::Glonass
                ]),
            }
        );
        let m2 = MaskFilter::from_str("eq:GPS,GAL,GLO").unwrap();
        assert_eq!(mask, m2);

        let mask = MaskFilter::from_str("neq:BDS").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::NotEquals,
                item: TargetItem::ConstellationItem(vec![Constellation::BeiDou]),
            }
        );
        let m2 = MaskFilter::from_str("neq:BDS").unwrap();
        assert_eq!(mask, m2);
    }
    #[test]
    fn mask_sv() {
        let mask = MaskFilter::from_str("eq:G08,  G09, R03").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::SvItem(vec![
                    Sv::from_str("G08").unwrap(),
                    Sv::from_str("G09").unwrap(),
                    Sv::from_str("R03").unwrap(),
                ]),
            }
        );
        let m2 = MaskFilter::from_str("eq:G08,G09,R03").unwrap();
        assert_eq!(mask, m2);

        let mask = MaskFilter::from_str("neq:G31").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::NotEquals,
                item: TargetItem::SvItem(vec![Sv::from_str("G31").unwrap(),]),
            }
        );
        let m2 = MaskFilter::from_str("neq:G31").unwrap();
        assert_eq!(mask, m2);
    }
    #[test]
    fn mask_observable() {
        let mask = MaskFilter::from_str("eq:L1C,S1C,D1P,C1W").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::ObservableItem(vec![
                    Observable::Phase("L1C".to_string()),
                    Observable::SSI("S1C".to_string()),
                    Observable::Doppler("D1P".to_string()),
                    Observable::PseudoRange("C1W".to_string()),
                ])
            }
        );
    }
    #[test]
    fn mask_orbit() {
        let mask = MaskFilter::from_str("eq:orb:iode").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::OrbitItem(vec![String::from("iode")])
            }
        );
    }
    #[test]
    fn mask_nav() {
        let mask = MaskFilter::from_str("eq:eph").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::NavFrameItem(vec![FrameClass::Ephemeris]),
            }
        );
        let mask = MaskFilter::from_str("eq:eph,ion").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::NavFrameItem(vec![
                    FrameClass::Ephemeris,
                    FrameClass::IonosphericModel
                ])
            }
        );
        let mask = MaskFilter::from_str("eq:lnav").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::NavMsgItem(vec![MsgType::LNAV]),
            }
        );
    }
}
