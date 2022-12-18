use super::{FilterItem, FilterItemError};
use crate::Epoch;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum MaskOperand {
    Above,
    StrictlyAbove,
    Below,
    StrictlyBelow,
}

impl std::str::FromStr for MaskOperand {
    type Err = MaskFilterError;
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
        } else {
            Err(MaskFilterError::UnknownOperand)
        }
    }
}

impl MaskOperand {
    pub(crate) const fn formatted_len(&self) -> usize {
        match &self {
            Self::Below | Self::Above => 1,
            Self::StrictlyBelow | Self::StrictlyAbove => 2,
        }
    }
}

/// Mask filter is one type of [super::Filter] with
/// an operand attached to it.
#[derive(Debug, Clone, PartialEq)]
pub struct MaskFilter<FilterItem> {
    pub operand: MaskOperand,
    pub item: FilterItem,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum MaskFilterError {
    #[error("unrecognized mask operand")]
    UnknownOperand,
    #[error("unrecognized filter item")]
    FilterItemError(#[from] FilterItemError),
}

impl std::str::FromStr for MaskFilter<FilterItem> {
    type Err = MaskFilterError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = content.trim();
        if let Ok(operand) = MaskOperand::from_str(content) {
            let offset = operand.formatted_len();
            let item = &content[offset..];
            let item = FilterItem::from_str(&content[offset..])?;
            Ok(Self { operand, item })
        } else {
            Err(MaskFilterError::UnknownOperand)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_mask_operand() {
        let operand = MaskOperand::from_str(">");
        assert_eq!(operand, Ok(MaskOperand::StrictlyAbove));

        let operand = MaskOperand::from_str(">=");
        assert_eq!(operand, Ok(MaskOperand::Above));

        let operand = MaskOperand::from_str("<");
        assert_eq!(operand, Ok(MaskOperand::StrictlyBelow));

        let operand = MaskOperand::from_str("<=");
        assert_eq!(operand, Ok(MaskOperand::Below));

        let operand = MaskOperand::from_str("<= 123");
        assert_eq!(operand, Ok(MaskOperand::Below));

        let operand = MaskOperand::from_str(">123.0");
        assert_eq!(operand, Ok(MaskOperand::StrictlyAbove));

        let operand = MaskOperand::from_str(">abcdefg");
        assert_eq!(operand, Ok(MaskOperand::StrictlyAbove));

        let operand = MaskOperand::from_str("!>");
        assert!(operand.is_err());
    }
    #[test]
    fn test_mask_filter() {
        let mask = MaskFilter::from_str("> 2020-01-14T00:31:55 UTC");
        assert_eq!(
            mask,
            Ok(MaskFilter {
                operand: MaskOperand::StrictlyAbove,
                item: FilterItem::EpochFilter(Epoch::from_str("2020-01-14T00:31:55 UTC").unwrap()),
            },)
        );

        let mask = MaskFilter::from_str("> JD 2452312.500372511 TAI");
        assert!(mask.is_ok());
    }
}
