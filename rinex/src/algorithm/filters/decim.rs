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
#[derive(Clone, Debug)]
pub enum DecimationType {
    /// Decimates Self by given factor.
    /// For example, if record contains epochs {e_0, e_1, .., e_k, ..., e_n}
    /// and we decimate by 2, we're left with epochs {e_0, e_2, ..., e_k, e_k+2, ..}.
    /// Header sampling interval (if any) is automatically adjusted.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::processing::Decimation;
    /// let mut rnx = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
    ///     .unwrap();
    /// assert_eq!(rnx.epochs().len(), 105);
    /// rnx.decim_by_ratio_mut(2); // reduce record size by 2
    /// assert_eq!(rnx.epochs().len(), 53);
    /// ```
    DecimByRatio(u32),
    /// Decimates Self by minimum epoch duration.
    /// Successive epochs |e_k+1 - e_k| < interval that do not fit
    /// within this minimal interval are discarded.
    /// Header sampling interval (if any) is automatically adjusted.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::processing::*; // Decimation
    /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// let initial_epochs = rinex.epochs();
    /// rinex.decim_by_interval_mut(Duration::from_seconds(10.0));
    /// assert_eq!(rinex.epochs(), initial_epochs); // unchanged, interval is too small
    /// rinex.decim_by_interval_mut(Duration::from_hours(1.0));
    /// assert_eq!(rinex.epochs().len(), initial_epochs.len()-2); // got rid of 2 epochs (15' and 25')
    /// ```
    DecimByInterval(Duration),
}

#[derive(Clone, Debug)]
pub struct DecimationFilter {
    /// Optional data subset
    target: Option<TargetItem>,
    /// Type of decimation filter
    pub dtype: DecimationType,
}

pub trait Decimate {
    fn decimate_by_ratio(&self, r: u32) -> Self;
    fn decimate_by_ratio_mut(&mut self, r: u32);
    fn decimate_by_interval(&self, dt: Duration) -> Self;
    fn decimate_by_interval_mut(&mut self, dt: Duration);
    fn decimate_match(&self, rhs: &Self) -> Self;
    fn decimate_match_mut(&mut self, rhs: &Self);
}

impl std::str::FromStr for DecimationFilter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.trim().split(":").collect();
        if let Ok(dt) = Duration::from_str(items[0].trim()) {
            Ok(Self {
                target: None,
                dtype: DecimationType::DecimByInterval(dt),
            })
        } else if let Ok(r) = u32::from_str_radix(items[0].trim(), 10) {
            Ok(Self {
                target: None,
                dtype: DecimationType::DecimByRatio(r),
            })
        } else {
            Err(Error::AttributeParsingError(items[0].to_string()))
        }
    }
}
