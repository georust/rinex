use crate::processing::TargetItem;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid mask target")]
    TargetError(#[from] crate::algorithm::target::Error),
    #[error("missing a mask operand")]
    MissingOperand,
    #[error("invalid mask operand")]
    InvalidOperand,
    #[error("invalid mask target \"{0}\"")]
    InvalidTarget(String),
    #[error("invalid mask description")]
    InvalidDescriptor,
}

pub trait Mask {
    fn mask(&self, mask: MaskFilter) -> Self;
    fn mask_mut(&mut self, mask: MaskFilter);
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
        } else if c.starts_with(">") {
            Ok(Self::GreaterThan)
        } else if c.starts_with("<=") {
            Ok(Self::LowerEquals)
        } else if c.starts_with("<") {
            Ok(Self::LowerThan)
        } else if c.starts_with("=") {
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
/// ```
/// use rinex::*;
/// use std::str::FromStr; // filter!
/// use rinex::processing::*; // .filter_mut()
/// // Grab a RINEX
/// let rinex = Rinex::from_file("../test_resources/OBS/V2/KOSG0010.95O")
///     .unwrap();
///
/// // Describe an [Hifitime::Epoch] for efficient datetime focus
/// let after = filter!(">2022-01-01T10:00:00 UTC");
/// let before = filter!("< 2022-01-01T10:00:00 UTC"); // whitespace tolerant
///
/// // logical Not() operation is supported on all mask operands.
/// // This may facilitate complex masking operations.
/// assert_eq!(before, !after);
///
/// // Any valid [Hifitime::Epoch] description is supported.
/// let equals = filter!("=JD 2960 TAI");
///
/// // Greater than ">" and lower than "<"
/// // truly apply to Epochs and Durations only,
/// // Whereas Equality masks ("=", "!=") apply to any data subsets.
/// let equals = filter!("=GPS,GLO");
///
/// // One exception exist for "Sv" items, for example with this:
/// let greater_than = filter!("> G08,R03");
///
/// // will retain PRN > 08 for GPS Constellation
/// // and PRN > 3 for Glonass Constellation.
/// let filtered = rinex.filter(greater_than);
///
/// // Focus on desired Observables, with an observable mask.
/// // This can apply to both OBS and Meteo RINEX.
/// let phase_mask = filter!("L1C,L2C"); // Equals operand is implied
/// let filtered = rinex.filter(phase_mask);
///
/// // Elevation angle filter can only apply to NAV RINEX
/// // content at the moment.
/// // In the future, this ops will be feasible if an OBS RINEX content
/// // is combined to NAV RINEX context.
/// let rinex = Rinex::from_file("../test_resources/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz")
///     .unwrap();
///
/// let mask = filter!("e > 10.0"); // strictly above 10° elevation
/// let filtered = rinex.filter(mask);
/// let mask = filter!("a >= 25.0"); // above 25° azimuth
/// let filtered = rinex.filter(mask);
///
/// // Apply an elevation range mask by combining two elevation masks
/// let mask = filter!("e <= 45.0"); // below 45°
/// let filtered = rinex.filter(mask);
///
/// // Retain only NAV RINEX Orbit fields you're interested in,
/// // with an Orbit mask:
/// let mask = filter!("iode,crs"); // retain only these fields,
///                             // notice: case insensitive,
///                             // notice: invalid orbit fields get dropped out
/// let filtered = rinex.filter(mask);
///
/// // Three other NAV RINEX specific cases exist
///
/// // [1] : Orbit fields mask
/// // To retain specific Orbit fields
///
/// // [2] : Message Type mask
/// // For example: retain Legacy NAV frames only.
/// // Any valid NavMessageType description works here
/// let mask = filter!("lnav"); // Eq() is implied
///
/// // [3] : Frame type mask
/// // For example: retain anything but Ephemeris and IonMessage frames with this.
/// let mask = filter!("!=eph,ion"); // Not an Eq(): we must specify the operand
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
        if cleanedup.len() < 3 {
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
            } else {
                if let Ok(op) = MaskOperand::from_str(&cleanedup[i..i + 1]) {
                    operand = Some(op.clone());
                    operand_offset = Some(i);
                    break;
                }
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

            let start = &cleanedup[..operand_offset];
            if start[0..1].eq("e") {
                // --> Elevation Mask case
                let float_offset = operand_offset + operand.formatted_len() +2;
                Ok(Self {
                    operand,
                    item: TargetItem::from_elevation(&cleanedup[float_offset..].trim())?,
                })
            } else if content[0..1].eq("a") {
                // --> Azimuth Mask case
                let float_offset = operand_offset + operand.formatted_len() +2;
                Ok(Self {
                    operand,
                    item: TargetItem::from_azimuth(&cleanedup[float_offset..].trim())?,
                })
            } else {
                // We're only left with SNR mask case
                let float_offset = operand_offset + operand.formatted_len() +2;
                if content[0..3].eq("snr") {
                    Ok(Self {
                        operand,
                        item: TargetItem::from_snr(&cleanedup[float_offset..].trim())?,
                    })
                } else {
                    Err(Error::InvalidTarget(
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
                item: TargetItem::from_str(&cleanedup[offset..].trim_start())?,
            })
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
        for (descriptor, opposite_desc) in vec![
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
                item: TargetItem::EpochItem(Epoch::from_str("2020-01-14T00:31:55 UTC").unwrap()),
            }
        );
        let mask = MaskFilter::from_str(">JD 2452312.500372511 TAI");
        assert!(mask.is_ok());
    }
    #[test]
    fn mask_elev() {
        for desc in vec!["e< 40.0", "e != 30", " e<40.0", " e < 40.0", " e > 120", " e >= 120", " e = 30"] {
            let mask = MaskFilter::from_str(desc);
            assert!(
                mask.is_ok(),
                "Failed to parse Elevation Mask Filter from \"{}\"",
                desc
            );
        }
    }
    #[test]
    fn mask_gnss() {
        for (descriptor, opposite_desc) in vec![
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
                item: TargetItem::ConstellationItem(vec![
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
                item: TargetItem::ConstellationItem(vec![Constellation::BeiDou]),
            }
        );
    }
    #[test]
    fn mask_sv() {
        for (descriptor, opposite_desc) in
            vec![(" = G01", "!= G01"), ("= R03,  G31", "!= R03,  G31")]
        {
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
                item: TargetItem::SvItem(vec![
                    Sv::from_str("G08").unwrap(),
                    Sv::from_str("G09").unwrap(),
                    Sv::from_str("R03").unwrap(),
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
                item: TargetItem::SvItem(vec![Sv::from_str("G31").unwrap(),]),
            }
        );
        let m2 = MaskFilter::from_str("!=G31").unwrap();
        assert_eq!(mask, m2);
    }
    #[test]
    fn mask_observable() {
        let mask = MaskFilter::from_str("=L1C,S1C,D1P,C1W").unwrap();
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
        let mask = MaskFilter::from_str("=iode").unwrap();
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
        let mask = MaskFilter::from_str("=eph").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::NavFrameItem(vec![FrameClass::Ephemeris]),
            }
        );
        let mask = MaskFilter::from_str("=eph,ion").unwrap();
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
        let mask = MaskFilter::from_str("=lnav").unwrap();
        assert_eq!(
            mask,
            MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::NavMsgItem(vec![MsgType::LNAV]),
            }
        );
    }
}
