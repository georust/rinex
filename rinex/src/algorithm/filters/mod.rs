mod mask;
use super::TargetItem;
use mask::{MaskFilter, MaskOperand};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("invalid filter description")]
	InvalidDescriptor,
	#[error("unknown filter type \"{0}\"")]
	UnknownFilterType(String),
	#[error("invalid mask filter \"{0}\"")]
	MaskError(String),
	#[error("invalid filter target")]
	TargetItemError(#[from] super::target::Error),
}

/// Smoothing Filter to smooth data subsets
pub enum SmoothingFilter {
	/// Hatch filter to smooth pseudo range observations
	HatchFilter,
}

/// Preprocessing filters /algorithms to preprocess RINEX data 
pub enum PreFilter {
	/// Mask filter is used to focus or remove a specific data subset
	Mask(mask::MaskFilter),
	/// Smoothing filter is used to smooth a data subset
	Smoothing(SmoothingFilter),
	// /// Interpolation filter for data subset (1D) interpolation
	// Interp(MaskFilter),
}

impl std::str::FromStr for PreFilter {
	type Err = Error;
	fn from_str(content: &str) -> Result<Self, Self::Err> {
		let items: Vec<&str> = content.split(":")
			.collect();
		if items.len() < 3 {
			return Err(Error::InvalidDescriptor);
		}
		if items[0].trim().eq("mask") {
			if let Ok(filt) = MaskFilter::from_str(&content[5..].trim()) {
				Ok(Self::Mask(filt))
			
			} else if let Ok(item) = TargetItem::from_str(&content[5..].trim()) {
				Ok(Self::Mask(
					MaskFilter {
						operand: MaskOperand::Equals,
						item,
					}
				))
			} else {
				Err(Error::MaskError(content[5..].trim().to_string()))
			}

		} else {
			Err(Error::UnknownFilterType(items[0].to_string()))
		}
	}
}

/*
pub trait Preprocessing {
	fn apply(&self, filt: PreFilter) -> Result<Self, AlgorithmError> where Self: Sized;
	fn apply_mut(&mut self, filt: PreFilter) -> Result<(), AlgorithmError>;
}*/

#[cfg(test)]
mod test {
	use super::*;
	use std::str::FromStr;
	#[test]
	fn algo_filter_maskfilter() {
		for desc in vec![
			"mask: gt: elev: 10.0",
			"mask: gnss: GPS",
			"mask: sv: G08, G09, G10",
			"mask: eq: gnss: GPS",
			"mask: eq: gnss: GLO, GAL",
			"mask: ineaeq: gnss: GLO, GAL",
			"mask: sv: G08, G09",
		] {
			let filt = PreFilter::from_str(desc);
			assert!(filt.is_ok());
		}
	}
}
