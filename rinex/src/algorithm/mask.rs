use super::{AlgorithmError, TargetItem};

/// MaskOperand describe how to apply a mask 
/// in related filter operation
#[derive(Debug, Clone, PartialEq)]
pub enum MaskOperand {
    Above,
    StrictlyAbove,
    Below,
    StrictlyBelow,
    Equal,
    NotEqual,
}

impl std::str::FromStr for MaskOperand {
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

impl MaskOperand {
    pub(crate) const fn formatted_len(&self) -> usize {
        match &self {
            Self::NotEqual | Self::Below | Self::Above => 2,
            Self::Equal | Self::StrictlyBelow | Self::StrictlyAbove => 1,
        }
    }
}

impl std::ops::Not for MaskOperand {
    type Output = MaskOperand;
    fn not(self) -> Self {
        match self {
            Self::Equal => Self::NotEqual,
            Self::NotEqual => Self::Equal,
            Self::StrictlyAbove => Self::Below,
            Self::Above => Self::StrictlyBelow,
            Self::StrictlyBelow => Self::Above,
            Self::Below => Self::StrictlyAbove,
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
/// let after = Mask::from_str("> 2022-01-01 10:00:00UTC")
///     .unwrap();
/// let before = Mask::from_str("<= 2022-01-01 10:00:00UTC")
///     .unwrap();
/// assert_eq!(before, !after); // logical not() is supported for all mask operands 
///
/// // the payload can be any valid Epoch Description,
/// // refer to [Hifitime::Epoch]
/// let equals = Mask::from_str("= JD 2960")
///     .unwrap();
/// assert_eq!(Mask::from_str("!= JD 2960").unwrap(), !equals);
///
/// // mask can apply to a lot of different data subsets,
/// // refer to the [TargetItem] definition,
///
/// // Greater than ">" and lower than "<" 
/// // truly apply to Epochs and Durations only,
/// // whereas Equality masks ("=", "!=") apply to any known item.
/// // One exception exist for "Sv" items, for example with this:
/// let greater_than = Mask::from_str(">= G08,R03")
///     .unwrap();
/// // would retain for GPS all vehicles with PRN>=08
/// // and PRN>=03 for Glonass
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    pub operand: MaskOperand,
    pub item: TargetItem,
}

impl std::str::FromStr for Mask {
    type Err = AlgorithmError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = content.trim();
        if content.len() < 3 {
            return Err(AlgorithmError::MalformedDescriptor);
        }
        if let Ok(operand) = MaskOperand::from_str(content) {
            let offset = operand.formatted_len();
            let item = &content[offset..];
            Ok(Self {
                operand,
                item: TargetItem::from_str(item)?,
            })
        } else {
            Err(AlgorithmError::UnknownOperand)
        }
    }
}

pub trait MaskFilter {
	/// Applies given filter to self.
	/// ```
	/// use rinex::prelude::*;
	/// use rinex::processing::*;
	/// // parse a RINEX file
	/// let rinex = Rinex::from_file("../test_resources/OBS/V3/")
	///		.unwrap();
    ///
    /// // design a mask
	/// let sv_filter: Mask::from_str("= GPS")
	///		.unwrap();
	/// let rinex = rinex.filter(sv_filter);
    ///
	/// // design a mask: case insensitive
	/// let sv_filter: Mask::from_str("= g08,g09")
	///		.unwrap();
	/// let rinex = rinex.filter(sv_filter);
    ///
	/// // whitespace is not mandatory,
	/// let phase_filter = Mask::from_str("=L1C,l2l,L2W")
	///		.unwrap();
	/// let rinex = rinex.filter(sv_filter);
    ///
    /// // if the descriptor is a integer number,
    /// // we expect it to be the numerical representation
    /// // of an epoch Flag
    /// let epoch_ok_mask = Mask::from_str("= 0")
    ///     .unwrap();
    /// let rinex = rinex.filter(epoch_ok_mask);
    ///
    /// // if the descriptor is a float number,
    /// // we consider it to be an elevation mask as of today.
    /// // Maybe in the future, we'll have options to describe
    /// // other types of fields or physics
    /// let elev_mask = Mask::from_str(">= 33.0") // elev mask in °
    ///     .unwrap();
    /// // Applying a mask like this can only work 
    /// // on Navigation Data: this is just an example.
    /// let rinex = rinex.filter(elev_mask);
	/// ```
    fn apply(&self, mask: Mask) -> Self;
	/// Mutable implementation, see [Filter::apply]
    fn apply_mut(&mut self, mask: Mask);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    use crate::prelude::*;
    use crate::navigation::{FrameClass, MsgType};
    #[test]
    fn test_mask_operand() {
        let operand = MaskOperand::from_str(">").unwrap();
        assert_eq!(operand, MaskOperand::StrictlyAbove);
        assert_eq!(!operand, MaskOperand::from_str("<=").unwrap());

        let operand = MaskOperand::from_str(">=").unwrap();
        assert_eq!(operand, MaskOperand::Above);
        assert_eq!(!operand, MaskOperand::from_str("<").unwrap());

        let operand = MaskOperand::from_str("<").unwrap();
        assert_eq!(operand, MaskOperand::StrictlyBelow);
        assert_eq!(!operand, MaskOperand::from_str(">=").unwrap());

        let operand = MaskOperand::from_str("<=").unwrap();
        assert_eq!(operand, MaskOperand::Below);
        assert_eq!(!operand, MaskOperand::from_str(">").unwrap());

        let operand = MaskOperand::from_str("<= 123").unwrap();
        assert_eq!(operand, MaskOperand::Below);
        assert_eq!(!operand, MaskOperand::from_str(">").unwrap());

        let operand = MaskOperand::from_str(">123.0").unwrap();
        assert_eq!(operand, MaskOperand::StrictlyAbove);
        assert_eq!(!operand, MaskOperand::from_str("<=").unwrap());

        let operand = MaskOperand::from_str(">abcdefg").unwrap();
        assert_eq!(operand, MaskOperand::StrictlyAbove);
        assert_eq!(!operand, MaskOperand::from_str("<=").unwrap());

        let operand = MaskOperand::from_str("!>");
        assert!(operand.is_err());
    }
    #[test]
    fn test_epoch_mask() {
        let mask = Mask::from_str("> 2020-01-14T00:31:55 UTC").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::StrictlyAbove,
                item: TargetItem::EpochItem(Epoch::from_str("2020-01-14T00:31:55 UTC").unwrap()),
            });
        let mask = Mask::from_str("> JD 2452312.500372511 TAI");
        assert!(mask.is_ok());
    }
    #[test]
    fn test_elev_mask() {
        let mask = Mask::from_str("< 40.0").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::StrictlyBelow,
                item: TargetItem::ElevationItem(40.0_f64),
            });
        let m2 = Mask::from_str("<40.0").unwrap();
        assert_eq!(mask, m2);
        let m2 = Mask::from_str("  < 40.0  ").unwrap();
        assert_eq!(mask, m2);

        let mask = Mask::from_str(">= 10.0").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::Above,
                item: TargetItem::ElevationItem(10.0_f64),
            });
        let m2 = Mask::from_str(">=10.0").unwrap();
        assert_eq!(mask, m2);
    }
    #[test]
    fn test_constell_mask() {
        let mask = Mask::from_str("=gnss:GPS").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::Equal,
                item: TargetItem::ConstellationItem(vec![Constellation::GPS]),
            });
        let m2 = Mask::from_str("= gnss: GPS").unwrap();
        assert_eq!(mask, m2);

        let mask = Mask::from_str("= gnss: GPS,GAL,GLO").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::Equal,
                item: TargetItem::ConstellationItem(vec![Constellation::GPS, Constellation::Galileo, Constellation::Glonass]),
            });
        let m2 = Mask::from_str("=gnss:GPS,GAL,GLO").unwrap();
        assert_eq!(mask, m2);
        
        let mask = Mask::from_str("!= BDS").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::NotEqual,
                item: TargetItem::ConstellationItem(vec![Constellation::BeiDou]),
            });
        let m2 = Mask::from_str("!=BDS").unwrap();
        assert_eq!(mask, m2);
    }
    #[test]
    fn test_sv_mask() {
        let mask = Mask::from_str("= sv:G08,  G09, R03").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::Equal,
                item: TargetItem::SvItem(vec![
                    Sv::from_str("G08").unwrap(),
                    Sv::from_str("G09").unwrap(),
                    Sv::from_str("R03").unwrap(),
                ]),
            });
        let m2 = Mask::from_str("=sv:G08,G09,R03").unwrap();
        assert_eq!(mask, m2);
        
        let mask = Mask::from_str("!= sv:G31").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::NotEqual,
                item: TargetItem::SvItem(vec![
                    Sv::from_str("G31").unwrap(),
                ]),
            });
        let m2 = Mask::from_str("!=G31").unwrap();
        assert_eq!(mask, m2);
    }
    #[test]
    fn test_obs_mask() {
        let mask = Mask::from_str("= L1C,S1C,D1P,C1W").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::Equal,
                item: TargetItem::ObservableItem(
                    vec![
                        Observable::Phase("L1C".to_string()),
                        Observable::SSI("S1C".to_string()),
                        Observable::Doppler("D1P".to_string()),
                        Observable::PseudoRange("C1W".to_string()),
                    ])
            });
    }
    #[test]
    fn test_orb_mask() {
        let mask = Mask::from_str("= iode").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::Equal,
                item: TargetItem::OrbitItem(vec![String::from("iode")])
            });
    }
    #[test]
    fn test_nav_mask() {
        let mask = Mask::from_str("= eph").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::Equal,
                item: TargetItem::NavFrameItem(
                    vec![FrameClass::Ephemeris]),
            });
        let mask = Mask::from_str("= eph,ion").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::Equal,
                item: TargetItem::NavFrameItem(
                    vec![FrameClass::Ephemeris, FrameClass::IonosphericModel])
            });
        let mask = Mask::from_str("= lnav").unwrap();
        assert_eq!(
            mask,
            Mask {
                operand: MaskOperand::Equal,
                item: TargetItem::NavMsgItem(vec![MsgType::LNAV]),
            });
    }
}
