use crate::{processing::TargetItem, Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown decimation target")]
    TargetError(#[from] crate::algorithm::target::Error),
    #[error("failed to parse decimation attribute \"{0}\"")]
    AttributeParsingError(String),
}

/// Decimation Filters type
#[derive(Clone, Debug, PartialEq)]
pub enum DecimationType {
    /// Decimates Dataset by given factor.
    DecimByRatio(u32),
    /// Decimates Dataset so sampling rate matches given duration
    DecimByInterval(Duration),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecimationFilter {
    /// Optional data subset
    pub target: Option<TargetItem>,
    /// Type of decimation filter
    pub dtype: DecimationType,
}

pub trait Decimate {
    /// Decimate by a constant ratio.
    /// For example, if we decimate epochs {e_0, e_1, .., e_k, ..., e_n}
    /// by 2, we get {e_0, e_2, ..., e_k, e_k+2, ..}.
    /// Header sampling interval (if any) is automatically updated.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::processing::*;
    /// let mut rnx = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
    ///     .unwrap();
    /// assert_eq!(rnx.epochs().len(), 105);
    /// assert_eq!(rnx.decimate_by_ratio(2).epochs().len(), 53);
    /// ```
    fn decimate_by_ratio(&self, r: u32) -> Self;
    /// [decimate_by_ratio] mutable implementation.
    fn decimate_by_ratio_mut(&mut self, r: u32);
    /// Decimate Dataset so sampling interval matches given duration.
    /// Successive epochs |e_k+1 - e_k| < interval that do not fit
    /// within this minimal interval are discarded.
    /// Header sampling interval (if any) is automatically update.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::processing::*; // Decimation
    /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    ///
    /// let initial_epochs = rinex.epochs();
    ///
    /// // reduce to 10s sampling interval
    /// rinex.decimate_by_interval_mut(Duration::from_seconds(10.0));
    /// assert_eq!(rinex.epochs(), initial_epochs); // unchanged: dt is too short
    ///
    /// // reduce to 1hour sampling interval
    /// rinex.decimate_by_interval_mut(Duration::from_hours(1.0));
    /// assert_eq!(rinex.epochs().len(), initial_epochs.len()-2);
    /// ```
    fn decimate_by_interval(&self, dt: Duration) -> Self;
    /// [decimate_by_interval] mutable implementation
    fn decimate_by_interval_mut(&mut self, dt: Duration);
    /// Decimate Dataset so sampling matches given `rhs` sampling.
    /// Both types must match.
    fn decimate_match(&self, rhs: &Self) -> Self;
    /// [decimate_match] mutable implementation
    fn decimate_match_mut(&mut self, rhs: &Self);
}

impl std::str::FromStr for DecimationFilter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.trim().split(":").collect();
        if let Ok(dt) = Duration::from_str(items[0].trim()) {
            Ok(Self {
                target: {
                    if items.len() > 1 {
                        let target = TargetItem::from_str(items[1].trim())?;
                        Some(target)
                    } else {
                        None // no subset description
                    }
                },
                dtype: DecimationType::DecimByInterval(dt),
            })
        } else if let Ok(r) = u32::from_str_radix(items[0].trim(), 10) {
            Ok(Self {
                target: {
                    if items.len() > 1 {
                        let target = TargetItem::from_str(items[1].trim())?;
                        Some(target)
                    } else {
                        None
                    }
                },
                dtype: DecimationType::DecimByRatio(r),
            })
        } else {
            Err(Error::AttributeParsingError(items[0].to_string()))
        }
    }
}
