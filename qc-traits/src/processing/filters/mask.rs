use crate::processing::{FilterItem, ItemError};
use thiserror::Error;

/// Mask filter parsing errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid mask item")]
    InvalidMaskitem(#[from] ItemError),
    #[error("missing mask operand")]
    MissingOperand,
    #[error("invalid mask operand")]
    InvalidOperand,
    #[error("invalid mask target \"{0}\"")]
    NonSupportedTarget(String),
    #[error("invalid mask description")]
    InvalidDescriptor,
}

/// Masking trait, to retain specific GNSS data subsets.  
/// This can be used to retain specific signals or [Constellation]s.
pub trait Mask {
    /// Apply mask filter to mutable self.
    fn mask_mut(&mut self, mask: MaskFilter);
    /// Immutable mask filter.
    fn mask(&self, mask: MaskFilter) -> Self;
}

/// MaskOperand describes how to apply a given mask
#[derive(Debug, Clone, PartialEq)]
pub enum MaskOperand {
    /// Greater than, is symbolized by ">".
    GreaterThan,
    /// Greater Equals, symbolized by ">=".
    GreaterEquals,
    /// Lower than, symbolized by "<"."
    LowerThan,
    /// Lower Equals, symbolized by "<=".
    LowerEquals,
    /// Equals, symbolized by "=".
    /// Equals operand is implied anytime the operand is omitted in the description.
    Equals,
    /// Not Equals, symbolized by "!=".
    NotEquals,
}

impl std::str::FromStr for MaskOperand {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        if c.starts_with(">=") {
            Ok(Self::GreaterEquals)
        } else if c.starts_with('>') {
            Ok(Self::GreaterThan)
        } else if c.starts_with("<=") {
            Ok(Self::LowerEquals)
        } else if c.starts_with('<') {
            Ok(Self::LowerThan)
        } else if c.starts_with('=') {
            Ok(Self::Equals)
        } else if c.starts_with("!=") {
            Ok(Self::NotEquals)
        } else {
            Err(Error::InvalidOperand)
        }
    }
}

impl MaskOperand {
    pub(crate) const fn formatted_len(&self) -> usize {
        match &self {
            Self::Equals | Self::GreaterThan | Self::LowerThan => 1,
            Self::NotEquals | Self::LowerEquals | Self::GreaterEquals => 2,
        }
    }
}

impl std::ops::Not for MaskOperand {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::Equals => Self::NotEquals,
            Self::NotEquals => Self::Equals,
            Self::GreaterEquals => Self::LowerEquals,
            Self::GreaterThan => Self::LowerThan,
            Self::LowerThan => Self::GreaterThan,
            Self::LowerEquals => Self::GreaterEquals,
        }
    }
}

/// Apply MaskFilters to focus on datasubsets you're interested in.
#[derive(Debug, Clone, PartialEq)]
pub struct MaskFilter {
    /// Item describes what subset we this [MaskFilter] applies to.
    pub item: FilterItem,
    /// Operand describes how to apply this [MaskFilter]
    pub operand: MaskOperand,
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
            // not permitted on operand mismatch
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
        let cleanedup = content.trim_start();
        if cleanedup.len() < 2 {
            /*
             * we're most likely unable to parsed both
             * an operand and a filter payload
             */
            return Err(Error::InvalidDescriptor);
        }

        let mut operand: Option<MaskOperand> = None;
        let mut operand_offset: Option<usize> = None;
        // In some cases, the target item comes first.
        // This allows more "human readable" descriptions,
        // but makes parsing a little harder.

        // Try to locate a mask operand within given content
        for i in 0..cleanedup.len() - 1 {
            if i < cleanedup.len() - 2 {
                if let Ok(op) = MaskOperand::from_str(&cleanedup[i..i + 2]) {
                    operand = Some(op.clone());
                    operand_offset = Some(i);
                    break;
                }
            } else if let Ok(op) = MaskOperand::from_str(&cleanedup[i..i + 1]) {
                operand = Some(op.clone());
                operand_offset = Some(i);
                break;
            }
        }

        let operand_omitted = operand_offset.is_none();

        let (operand, operand_offset): (MaskOperand, usize) = match operand_offset.is_some() {
            true => (operand.unwrap(), operand_offset.unwrap()),
            false => {
                /*
                 * Operand was not found, it's either omitted and Eq() is implied,
                 * or this parser will soon fail due to faulty content
                 */
                (MaskOperand::Equals, 0)
            },
        };

        if operand_offset > 0 {
            // Some characters exist between .start() and identified operand.
            // Type guessing for filter target will not work.
            // This only exits for Elevation Angle, Azimuth Angle and SNR masks at the moment.

            // Simply due to the fact that the operand is located
            // after the identifier, in those cases

            let start = &cleanedup[..operand_offset];
            if start[0..1].eq("e") {
                // --> Elevation Mask case
                let float_offset = operand_offset + operand.formatted_len() + 2;
                Ok(Self {
                    operand,
                    item: FilterItem::from_elevation(cleanedup[float_offset..].trim())?,
                })
            } else if content[0..1].eq("a") {
                // --> Azimuth Mask case
                let float_offset = operand_offset + operand.formatted_len() + 2;
                Ok(Self {
                    operand,
                    item: FilterItem::from_azimuth(cleanedup[float_offset..].trim())?,
                })
            } else {
                // We're only left with SNR mask case
                let float_offset = operand_offset + operand.formatted_len() + 2;
                if content[0..3].eq("snr") {
                    Ok(Self {
                        operand,
                        item: FilterItem::from_snr(cleanedup[float_offset..].trim())?,
                    })
                } else {
                    Err(Error::NonSupportedTarget(
                        cleanedup[..operand_offset].to_string(),
                    ))
                }
            }
        } else {
            // Descriptor starts with mask operand.
            // Filter target type guessing is possible.
            let offset: usize = match operand_omitted {
                false => operand_offset + operand.formatted_len(),
                true => 0,
            };

            Ok(Self {
                operand,
                item: FilterItem::from_str(cleanedup[offset..].trim_start())?,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use gnss_rs::prelude::{Constellation, SV};
    use hifitime::Epoch;
    use std::str::FromStr;
    #[test]
    fn mask_operand() {
        for (descriptor, opposite_desc) in [
            (">=", "<="),
            (">", "<"),
            ("=", "!="),
            ("<", ">"),
            ("<=", ">="),
        ] {
            let operand = MaskOperand::from_str(descriptor);
            assert!(
                operand.is_ok(),
                "{} \"{}\"",
                "Failed to parse MaskOperand from",
                descriptor
            );
            let opposite = MaskOperand::from_str(opposite_desc);
            assert!(
                opposite.is_ok(),
                "{} \"{}\"",
                "Failed to parse MaskOperand from",
                opposite_desc
            );
            assert_eq!(!operand.unwrap(), opposite.unwrap(), "MaskOperand::Not()");
        }

        let operand = MaskOperand::from_str("a");
        assert!(
            operand.is_err(),
            "Parsed unexpectedly \"{}\" MaskOperand correctly",
            "a"
        );
    }
    #[test]
    fn mask_epoch() {
        let mask = MaskFilter::from_str(">2020-01-14T00:31:55 UTC").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::GreaterThan,
                item: FilterItem::EpochItem(Epoch::from_str("2020-01-14T00:31:55 UTC").unwrap()),
            }
        );
        let mask = MaskFilter::from_str(">JD 2452312.500372511 TAI");
        assert!(mask.is_ok());
    }
    #[test]
    fn mask_elev() {
        for (desc, valid) in [
            ("e>1.0", true),
            ("e< 40.0", true),
            ("e != 30", true),
            (" e<40.0", true),
            (" e < 40.0", true),
            (" e > 120", false),
            (" e >= 120", false),
            (" e = 30", true),
        ] {
            let mask = MaskFilter::from_str(desc);
            assert_eq!(
                mask.is_ok(),
                valid,
                "failed to parse elevation mask filter \"{}\"",
                desc
            );
        }
    }
    #[test]
    fn mask_gnss() {
        for (descriptor, opposite_desc) in [
            (" = GPS", "!= GPS"),
            ("= GAL,GPS", "!= GAL,GPS"),
            (" =GLO,GAL", "!=  GLO,GAL"),
        ] {
            let mask = MaskFilter::from_str(descriptor);
            assert!(
                mask.is_ok(),
                "Unable to parse MaskFilter from \"{}\"",
                descriptor
            );
            let opposite = MaskFilter::from_str(opposite_desc);
            assert!(
                opposite.is_ok(),
                "Unable to parse MaskFilter from \"{}\"",
                opposite_desc
            );
            assert_eq!(!mask.unwrap(), opposite.unwrap(), "{}", "MaskFilter::Not()");
        }

        let mask = MaskFilter::from_str("=GPS,GAL,GLO").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: FilterItem::ConstellationItem(vec![
                    Constellation::GPS,
                    Constellation::Galileo,
                    Constellation::Glonass
                ]),
            }
        );

        let mask = MaskFilter::from_str("!=BDS").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::NotEquals,
                item: FilterItem::ConstellationItem(vec![Constellation::BeiDou]),
            }
        );
    }
    #[test]
    fn mask_sv() {
        for (descriptor, opposite_desc) in [(" = G01", "!= G01"), ("= R03,  G31", "!= R03,  G31")] {
            let mask = MaskFilter::from_str(descriptor);
            assert!(
                mask.is_ok(),
                "Unable to parse MaskFilter from \"{}\"",
                descriptor
            );
            let opposite = MaskFilter::from_str(opposite_desc);
            assert!(
                opposite.is_ok(),
                "Unable to parse MaskFilter from \"{}\"",
                opposite_desc
            );
            assert_eq!(!mask.unwrap(), opposite.unwrap(), "{}", "MaskFilter::Not()");
        }

        let mask = MaskFilter::from_str("=G08,  G09, R03").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: FilterItem::SvItem(vec![
                    SV::from_str("G08").unwrap(),
                    SV::from_str("G09").unwrap(),
                    SV::from_str("R03").unwrap(),
                ]),
            }
        );
        let m2 = MaskFilter::from_str("G08,G09,R03").unwrap();
        assert_eq!(mask, m2);

        let mask = MaskFilter::from_str("!=G31").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::NotEquals,
                item: FilterItem::SvItem(vec![SV::from_str("G31").unwrap(),]),
            }
        );
        let m2 = MaskFilter::from_str("!=G31").unwrap();
        assert_eq!(mask, m2);
    }
    #[test]
    fn mask_complex() {
        let mask = MaskFilter::from_str("=L1C,S1C,D1P,C1W").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: FilterItem::ComplexItem(vec![
                    "L1C".to_string(),
                    "S1C".to_string(),
                    "D1P".to_string(),
                    "C1W".to_string()
                ])
            }
        );
    }
}
