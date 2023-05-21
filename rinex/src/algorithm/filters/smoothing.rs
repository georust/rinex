use crate::{processing::TargetItem, Duration};
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
    InvalidDescriptionError(String),
    #[error("unknown smoothing filter \"{0}\"")]
    UnknownFilter(String),
    #[error("unknown smoothing target")]
    TargetError(#[from] crate::algorithm::target::Error),
    #[error("failed to parse duration")]
    DurationParsingError(#[from] hifitime::Errors),
}

impl std::str::FromStr for SmoothingFilter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.trim().split(":").collect();
        if items[0].trim().eq("hatch") {
            if items.len() > 1 {
                let target = TargetItem::from_str(items[1].trim())?;
                Ok(Self {
                    target: Some(target),
                    stype: SmoothingType::Hatch,
                })
            } else {
                Ok(Self {
                    target: None,
                    stype: SmoothingType::Hatch,
                })
            }
        } else if items[0].trim().eq("mov") {
            if items.len() < 2 {
                return Err(Error::InvalidDescriptionError(format!("{:?}", items)));
            }
            let dt = Duration::from_str(items[1].trim())?;
            if items.len() > 2 {
                let target = TargetItem::from_str(items[2].trim())?;
                Ok(Self {
                    target: Some(target),
                    stype: SmoothingType::MovingAverage(dt),
                })
            } else {
                Ok(Self {
                    target: None,
                    stype: SmoothingType::MovingAverage(dt),
                })
            }
        } else {
            Err(Error::UnknownFilter(items[0].to_string()))
        }
    }
}

pub trait Smooth {
    /// Applies mov average filter to self
    fn moving_average(&self, window: Duration, target: Option<TargetItem>) -> Self;
    /// Moving average mutable implementation
    fn moving_average_mut(&mut self, window: Duration, target: Option<TargetItem>);
    /// Applies a Hatch smoothing filter to Pseudo Range observations
    fn hatch_smoothing(&self, target: Option<TargetItem>) -> Self;
    /// Hatch filter mutable implementation
    fn hatch_smoothing_mut(&mut self, target: Option<TargetItem>);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn from_str() {
        for desc in vec!["hatch", "hatch:C1C", "hatch:c1c,c2p"] {
            let filter = SmoothingFilter::from_str(desc);
            assert!(
                filter.is_ok(),
                "smoothing_filter::from_str() failed on \"{}\"",
                desc
            );
        }
        for desc in vec![
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
