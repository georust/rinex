use std::str::FromStr;
use thiserror::Error;

use std::collections::BTreeMap;
use strum_macros::EnumString;

use crate::{
    epoch, merge,
    merge::Merge,
    prelude::*,
    prelude::{Duration, SV},
    split,
    split::Split,
    version::Version,
};

#[cfg(feature = "processing")]
use qc_traits::processing::{
    DecimationFilter, DecimationFilterType, FilterItem, MaskFilter, MaskOperand,
};


impl std::fmt::Display for ClockType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(sv) = self.as_sv() {
            f.write_str(&sv.to_string())?
        } else if let Some(station) = self.as_station() {
            f.write_str(&station)?
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
/// Clocks file parsing & identification errors
pub enum Error {
    #[error("unknown data code \"{0}\"")]
    UnknownDataCode(String),
    #[error("failed to parse epoch")]
    EpochParsingError(#[from] epoch::ParsingError),
    #[error("failed to parse # of data fields")]
    DataFieldsParsing,
    #[error("failed to parse clock profile")]
    ClockProfileParsing,
    #[error("failed to identify observable")]
    ParseObservableError(#[from] strum::ParseError),
    #[error("failed to write data")]
    WriterIoError(#[from] std::io::Error),
}



/// Clock data observables
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ClockProfileType {
    /// Data analysis results for receiver clocks
    /// derived from a set of network receivers and satellites
    AR,
    /// Data analysis results for satellites clocks
    /// derived from a set of network receivers and satellites
    AS,
    /// Calibration measurements for a single GNSS receiver
    CR,
    /// Discontinuity measurements for a single GNSS receiver
    DR,
    /// Broadcast SV clocks monitor measurements
    MS,
}

impl std::fmt::Display for ClockProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AR => f.write_str("AR"),
            Self::AS => f.write_str("AS"),
            Self::CR => f.write_str("CR"),
            Self::DR => f.write_str("DR"),
            Self::MS => f.write_str("MS"),
        }
    }
}

use crate::merge::merge_mut_option;

impl Merge for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        for (rhs_epoch, rhs_content) in rhs.iter() {
            if let Some(lhs_content) = self.get_mut(rhs_epoch) {
                for (rhs_key, rhs_prof) in rhs_content.iter() {
                    if let Some(lhs_prof) = lhs_content.get_mut(rhs_key) {
                        // enhance only, if possible
                        merge_mut_option(&mut lhs_prof.drift, &rhs_prof.drift);
                        merge_mut_option(&mut lhs_prof.drift_dev, &rhs_prof.drift_dev);
                        merge_mut_option(&mut lhs_prof.drift_change, &rhs_prof.drift_change);
                        merge_mut_option(
                            &mut lhs_prof.drift_change_dev,
                            &rhs_prof.drift_change_dev,
                        );
                    } else {
                        lhs_content.insert(rhs_key.clone(), rhs_prof.clone());
                    }
                }
            } else {
                self.insert(*rhs_epoch, rhs_content.clone());
            }
        }
        Ok(())
    }
}

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if k <= &epoch {
                    Some((*k, v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k > &epoch {
                    Some((*k, v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((r0, r1))
    }
    fn split_dt(&self, _duration: Duration) -> Result<Vec<Self>, split::Error> {
        Ok(Vec::new())
    }
}

#[cfg(feature = "processing")]
pub(crate) fn clock_mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e == *epoch),
            FilterItem::ConstellationItem(mask) => {
                rec.retain(|_, data| {
                    data.retain(|sysclk, _| {
                        if let Some(sv) = sysclk.clock_type.as_sv() {
                            mask.contains(&sv.constellation)
                        } else {
                            false
                        }
                    });
                    !data.is_empty()
                });
            },
            _ => {}, // FilterItem::
        },
        MaskOperand::NotEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e != *epoch),
            _ => {}, // FilterItem::
        },
        MaskOperand::GreaterEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e >= *epoch),
            _ => {}, // FilterItem::
        },
        MaskOperand::GreaterThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e > *epoch),
            _ => {}, // FilterItem::
        },
        MaskOperand::LowerEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e <= *epoch),
            _ => {}, // FilterItem::
        },
        MaskOperand::LowerThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e < *epoch),
            _ => {}, // FilterItem::
        },
    }
}

#[cfg(feature = "processing")]
pub(crate) fn clock_decim_mut(rec: &mut Record, f: &DecimationFilter) {
    if f.item.is_some() {
        todo!("targetted decimation not supported yet");
    }
    match f.filter {
        DecimationFilterType::Modulo(r) => {
            let mut i = 0;
            rec.retain(|_, _| {
                let retained = (i % r) == 0;
                i += 1;
                retained
            });
        },
        DecimationFilterType::Duration(interval) => {
            let mut last_retained = Option::<Epoch>::None;
            rec.retain(|e, _| {
                if let Some(last) = last_retained {
                    let dt = *e - last;
                    if dt >= interval {
                        last_retained = Some(*e);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(*e);
                    true // always retain 1st epoch
                }
            });
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::SV;
    use crate::version::Version;
    use std::str::FromStr;
    #[test]
    fn parse_clk_v2_epoch() {
        for (descriptor, epoch, key, profile) in [
            (
                "AS R20  2019 01 08 00 03 30.000000  1   -0.364887538519E-03",
                Epoch::from_str("2019-01-08T00:03:30 GPST").unwrap(),
                ClockKey {
                    clock_type: ClockType::SV(SV::from_str("R20").unwrap()),
                    profile_type: ClockProfileType::AS,
                },
                ClockProfile {
                    bias: -0.364887538519E-03,
                    bias_dev: None,
                    drift: None,
                    drift_change: None,
                    drift_dev: None,
                    drift_change_dev: None,
                },
            ),
            (
                "AS R18  2019 01 08 10 00  0.000000  2    0.294804625338E-04  0.835484069663E-11",
                Epoch::from_str("2019-01-08T10:00:00 GPST").unwrap(),
                ClockKey {
                    clock_type: ClockType::SV(SV::from_str("R18").unwrap()),
                    profile_type: ClockProfileType::AS,
                },
                ClockProfile {
                    bias: 0.294804625338E-04,
                    bias_dev: Some(0.835484069663E-11),
                    drift: None,
                    drift_dev: None,
                    drift_change: None,
                    drift_change_dev: None,
                },
            ),
            (
                "AR PIE1 2019 01 08 00 04  0.000000  1   -0.434275035628E-03",
                Epoch::from_str("2019-01-08T00:04:00 GPST").unwrap(),
                ClockKey {
                    clock_type: ClockType::Station("PIE1".to_string()),
                    profile_type: ClockProfileType::AR,
                },
                ClockProfile {
                    bias: -0.434275035628E-03,
                    bias_dev: None,
                    drift: None,
                    drift_dev: None,
                    drift_change: None,
                    drift_change_dev: None,
                },
            ),
            (
                "AR IMPZ 2019 01 08 00 00  0.000000  2   -0.331415119107E-07  0.350626190546E-10",
                Epoch::from_str("2019-01-08T00:00:00 GPST").unwrap(),
                ClockKey {
                    clock_type: ClockType::Station("IMPZ".to_string()),
                    profile_type: ClockProfileType::AR,
                },
                ClockProfile {
                    bias: -0.331415119107E-07,
                    bias_dev: Some(0.350626190546E-10),
                    drift: None,
                    drift_dev: None,
                    drift_change: None,
                    drift_change_dev: None,
                },
            ),
        ] {
            let (parsed_e, parsed_k, parsed_prof) =
                parse_epoch(Version { minor: 0, major: 2 }, descriptor, TimeScale::GPST)
                    .unwrap_or_else(|_| panic!("failed to parse \"{}\"", descriptor));

            assert_eq!(parsed_e, epoch, "parsed wrong epoch");
            assert_eq!(parsed_k, key, "parsed wrong clock id");
            assert_eq!(parsed_prof, profile, "parsed wrong clock data");
        }
    }
    #[test]
    fn parse_clk_v3_epoch() {
        for (descriptor, epoch, key, profile) in [
            (
                "AR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01
    -0.123456789012E+02 -0.123456789012E+03 -0.123456789012E+04 -0.123456789012E+05",
                Epoch::from_str("1994-07-14T20:59:00 GPST").unwrap(),
                ClockKey {
                    clock_type: ClockType::Station("AREQ".to_string()),
                    profile_type: ClockProfileType::AR,
                },
                ClockProfile {
                    bias: -0.123456789012E+00,
                    bias_dev: Some(-0.123456789012E+01),
                    drift: Some(-0.123456789012E+02),
                    drift_dev: Some(-0.123456789012E+03),
                    drift_change: Some(-0.123456789012E+04),
                    drift_change_dev: Some(-0.123456789012E+05),
                },
            ),
            (
                "AS G16  1994 07 14 20 59  0.000000  2   -0.123456789012E+00 -0.123456789012E+01",
                Epoch::from_str("1994-07-14T20:59:00 GPST").unwrap(),
                ClockKey {
                    clock_type: ClockType::SV(SV::from_str("G16").unwrap()),
                    profile_type: ClockProfileType::AS,
                },
                ClockProfile {
                    bias: -0.123456789012E+00,
                    bias_dev: Some(-0.123456789012E+01),
                    drift: None,
                    drift_dev: None,
                    drift_change: None,
                    drift_change_dev: None,
                },
            ),
            (
                "CR USNO 1994 07 14 20 59  0.000000  2   -0.123456789012E+00 -0.123456789012E+01",
                Epoch::from_str("1994-07-14T20:59:00 GPST").unwrap(),
                ClockKey {
                    clock_type: ClockType::Station("USNO".to_string()),
                    profile_type: ClockProfileType::CR,
                },
                ClockProfile {
                    bias: -0.123456789012E+00,
                    bias_dev: Some(-0.123456789012E+01),
                    drift: None,
                    drift_dev: None,
                    drift_change: None,
                    drift_change_dev: None,
                },
            ),
            (
                "DR USNO 1994 07 14 20 59  0.000000  2   -0.123456789012E+00 -0.123456789012E+01
    -0.123456789012E-03 -0.123456789012E-04",
                Epoch::from_str("1994-07-14T20:59:00 GPST").unwrap(),
                ClockKey {
                    clock_type: ClockType::Station("USNO".to_string()),
                    profile_type: ClockProfileType::DR,
                },
                ClockProfile {
                    bias: -0.123456789012E+00,
                    bias_dev: Some(-0.123456789012E+01),
                    drift: Some(-0.123456789012E-03),
                    drift_dev: Some(-0.123456789012E-04),
                    drift_change: None,
                    drift_change_dev: None,
                },
            ),
        ] {
            let (parsed_e, parsed_k, parsed_prof) =
                parse_epoch(Version { minor: 0, major: 2 }, descriptor, TimeScale::GPST)
                    .unwrap_or_else(|_| panic!("failed to parse \"{}\"", descriptor));

            assert_eq!(parsed_e, epoch, "parsed wrong epoch");
            assert_eq!(parsed_k, key, "parsed wrong clock id");
            assert_eq!(parsed_prof, profile, "parsed wrong clock data");
        }
    }
}
