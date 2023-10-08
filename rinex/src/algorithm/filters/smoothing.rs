use crate::{preprocessing::TargetItem, Duration};
use thiserror::Error;

/// Supported Smoothing Filters
#[derive(Debug, Clone, PartialEq)]
pub enum SmoothingType {
    /// Moving average filter
    MovingAverage(Duration),
    /// Hatch filter: Pseudo range specific smoothing method
    Hatch,
}

/// Smoothing Filter to smooth data subsets
#[derive(Debug, Clone, PartialEq)]
pub struct SmoothingFilter {
    /// Possible targeted subset to narrow down filter's application.
    /// When undefined, the filter applies to entire dataset
    pub target: Option<TargetItem>,
    /// Type of smoothing to apply
    pub stype: SmoothingType,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid description \"{0}\"")]
    InvalidDescription(String),
    #[error("unknown smoothing filter \"{0}\"")]
    UnknownFilter(String),
    #[error("invalid target")]
    InvalidTarget(#[from] crate::algorithm::target::Error),
    #[error("failed to parse duration")]
    DurationParsing(#[from] hifitime::Errors),
}

impl std::str::FromStr for SmoothingFilter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.trim().split(':').collect();
        if items[0].trim().eq("hatch") {
            Ok(Self {
                target: {
                    if items.len() > 1 {
                        let target = TargetItem::from_str(items[1].trim())?;
                        Some(target)
                    } else {
                        None // no subset description
                    }
                },
                stype: SmoothingType::Hatch,
            })
        } else if items[0].trim().eq("mov") {
            if items.len() < 2 {
                return Err(Error::InvalidDescription(format!("{:?}", items)));
            }
            let dt = Duration::from_str(items[1].trim())?;
            Ok(Self {
                target: {
                    if items.len() > 2 {
                        let target = TargetItem::from_str(items[2].trim())?;
                        Some(target)
                    } else {
                        None // no data subset
                    }
                },
                stype: SmoothingType::MovingAverage(dt),
            })
        } else {
            Err(Error::UnknownFilter(items[0].to_string()))
        }
    }
}

pub trait Smooth {
    /// Applies mov average filter to self
    fn moving_average(&self, window: Duration) -> Self;
    /// Moving average mutable implementation
    fn moving_average_mut(&mut self, window: Duration);
    /// Applies a Hatch smoothing filter to Pseudo Range observations
    fn hatch_smoothing(&self) -> Self;
    /// Hatch filter mutable implementation
    fn hatch_smoothing_mut(&mut self);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn from_str() {
        for desc in ["hatch", "hatch:C1C", "hatch:c1c,c2p"] {
            let filter = SmoothingFilter::from_str(desc);
            assert!(
                filter.is_ok(),
                "smoothing_filter::from_str() failed on \"{}\"",
                desc
            );
        }
        for desc in [
            "mov:10 min",
            "mov:1 hour",
            "mov:10 min:clk",
            "mov:10 hour:clk",
        ] {
            let filter = SmoothingFilter::from_str(desc);
            assert!(
                filter.is_ok(),
                "smoothing_filter::from_str() failed on \"{}\"",
                desc
            );
        }
    }
}
