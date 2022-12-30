mod mask;
mod smoothing;

use super::TargetItem;
pub use mask::{MaskFilter, MaskOperand};
pub use smoothing::{SmoothingFilter};

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
	#[error("failed to apply filter")]
	FilterError,
}

/// Preprocessing filters, to process RINEX data 
/// prior analysis
#[derive(Debug, Clone)]
pub enum Filter {
	/// Mask filter is used to focus or remove a specific data subset
	Mask(mask::MaskFilter),
	/// Smoothing filter is used to smooth a data subset
	Smoothing(SmoothingFilter),
	// /// Interpolation filter for data subset (1D) interpolation
	// Interp(MaskFilter),
}

impl From<MaskFilter> for Filter {
	fn from(mask: mask::MaskFilter) -> Self {
		Self::Mask(mask)
	}
}

impl std::str::FromStr for Filter {
	type Err = Error;
	fn from_str(content: &str) -> Result<Self, Self::Err> {
		let items: Vec<&str> = content.split(":")
			.collect();
		if items.len() < 2 {
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
		} else if items[0].trim().eq("smooth") {
			if let Ok(filt) = SmoothingFilter::from_str(&content[7..].trim()) {
				Ok(Self::Smoothing(filt))
			} else {
				Err(Error::MaskError(content[7..].trim().to_string()))
			}

		} else {
			Err(Error::UnknownFilterType(items[0].to_string()))
		}
	}
}

pub trait Preprocessing {
	fn filter(&self, filt: Filter) -> Self;
	fn filter_mut(&mut self, filt: Filter);
}

#[cfg(test)]
mod test {
	use super::*;
	use std::str::FromStr;
	#[test]
	fn algo_filter_maskfilter() {
		for desc in vec![
			"mask:gt: 10.0",
			"mask:eq:GPS",
			"mask:neq: GPS",
			"mask:eq:G08, G09, G10",
			"mask:neq:GPS, GAL",
			"mask:gt: G08, G09",
			"mask:eq:GPS",
			"mask:eq:GPS, GAL",
			"mask:eq:G08, G09",
		] {
			let filt = Filter::from_str(desc);
			assert!(filt.is_ok(), "Filter::from_str error on \"{}\"", desc);
		}
	}
	#[test]
	fn algo_filter_omitted_operands() {
		for desc in vec![
			"mask:10.0",
			"mask:10.0, 13.0",
			"mask:GPS",
			"mask:GPS,GAL",
			"mask:G08, G09, G10",
		] {
			let filt = Filter::from_str(desc);
			assert!(filt.is_ok(), "Filter::from_str error on \"{}\"", desc);
		}
	}
}