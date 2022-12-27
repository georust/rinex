use crate::Duration;

/// Smoothing Filter to smooth data subsets
#[derive(Debug, PartialEq, Clone)]
pub enum SmoothingFilter {
	/// Hatch filter to smooth pseudo range observations.
	/// Only applies to this subset
	HatchFilter,
	/// Applies Window Average filter,
	/// either to entire set, or to specific subset
	MovingAverage(Option<Duration>)
}

use thiserror::Error;
use crate::processing::TargetItem;

#[derive(Error, Debug)]
pub enum Error {
	#[error("unknown smoothing filter \"{0}\"")]
	UnknownFilter(String),
	#[error("unknown smoothing target")]
	TargetError(#[from] crate::algorithm::target::Error),
	#[error("failed to parse duration \"{0}\"")]
	DurationParsingError(String),
}

impl std::str::FromStr for SmoothingFilter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        if c.eq("hatch") {
            Ok(Self::HatchFilter)
        } else if c.starts_with("mov:") {
			/*
			 * Moving Average with specified window guessing
			 */
			 if let Ok(dt) = Duration::from_str(&c[4..].trim()) {
				Ok(Self::MovingAverage(Some(dt)))
			 } else {
			 	Err(Error::DurationParsingError(c.to_string()))
			}
		} else if c.starts_with("mov") {
			/*
			 * Moving Average with smart window guessing
			 */
			Ok(Self::MovingAverage(None))
        } else {
            Err(Error::UnknownFilter(c.to_string()))
        }
	}
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn algo_filter_smoothing() {
        let filter = SmoothingFilter::from_str("hatch")
			.unwrap();
        assert_eq!(filter, SmoothingFilter::HatchFilter);
        
		let filter = SmoothingFilter::from_str("mov")
			.unwrap();
        assert_eq!(filter, SmoothingFilter::MovingAverage(None));
        
		let filter = SmoothingFilter::from_str("test");
		assert!(filter.is_err());
	}
}
