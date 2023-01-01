use crate::{
	Duration,
	processing::TargetItem,
};
use thiserror::Error;

/// Supported Smoothing Filters
#[derive(Debug, Clone, PartialEq)]
pub enum SmoothingType {
	/// Hatch filter, only applies to pseudo range observations
	Hatch,
	// /// Applies Window Average filter,
	// /// either to entire set, or to specific subset
	// MovingAverage(Option<Duration>)
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
				stype: SmoothingType::Hatch,
			})
		} else {
            Err(Error::UnknownFilter(items[0].to_string()))
        }
	}
}

pub trait Smooth<T> {
	/// Applies a Hatch smoothing filter to Pseudo Range observations
	fn hatch_smoothing(&self) -> Self;
	/// Applies a Hatch smoothing filter to Pseudo Range observations
	fn hatch_smoothing_mut(&mut self);
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
				stype: SmoothingType::Hatch,
			});
	}
}
