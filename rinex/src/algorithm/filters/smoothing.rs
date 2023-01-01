use crate::{
	Duration,
	processing::TargetItem,
};
use thiserror::Error;

/// Known Smoothing Filters
#[derive(Debug, Clone, PartialEq)]
pub enum SmoothingType {
	/// Hatch filter, only applies to pseudo range observations
	Hatch,
	/// Applies Window Average filter,
	/// either to entire set, or to specific subset
	MovingAverage(Option<Duration>)
}

/// Smoothing Filter to smooth data subsets
#[derive(Debug, Clone, PartialEq)]
pub struct SmoothingFilter {
	/// Possible targeted subset to narrow down filter's application.
	/// When undefined, the filter applies to entire dataset
	pub target: Option<TargetItem>,
	/// Type of smoothing to apply
	pub smooth_type: SmoothingType,
}

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
        let items : Vec<&str> = content.trim().split(":").collect();
		if items[0].trim().eq("hatch") {
            Ok(Self {
				target: None,
				smooth_type: SmoothingType::Hatch,
			})
        
		/*} else if c.starts_with("mov:") {
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
			Ok(Self::MovingAverage(None)) */
        
		} else {
            Err(Error::UnknownFilter(items[0].to_string()))
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
        assert_eq!(filter, 
			SmoothingFilter {
				target: None,
				smooth_type: SmoothingType::Hatch,
			});
	}
}
