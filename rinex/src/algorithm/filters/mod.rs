mod decim;
mod interp;
mod mask;
mod smoothing;

use super::TargetItem;
pub use decim::{Decimate, DecimationFilter, DecimationType};
pub use interp::{InterpFilter, InterpMethod, Interpolate};
pub use mask::{Mask, MaskFilter, MaskOperand};
pub use smoothing::{Smooth, SmoothingFilter, SmoothingType};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid filter description")]
    InvalidDescriptor,
    #[error("unknown filter type \"{0}\"")]
    UnknownFilterType(String),
    #[error("invalid mask filter \"{0}\"")]
    MaskError(String),
    #[error("invalid decim filter \"{0}\"")]
    DecimError(String),
    #[error("invalid filter target")]
    TargetItemError(#[from] super::target::Error),
    #[error("failed to apply filter")]
    FilterError,
}

/// Preprocessing filters, to preprocess RINEX data
/// prior further analysis.
/// 
/// Apply masks to your dataset, to focus (retain)
/// on the subsets you're interested in
/// ```
/// let rinex = Rinex::from_file("../test_resource/OBS/V2/KOSG0010.95O")
///     .unwrap();
/// // retain specific Space Vehicules with an Sv mask
/// let mask = Filter::from_str("g01,g02,g03, g05")
///     .unwrap();
/// let masked = rinex.apply(mask);
/// // we're left with "G01", "G02", "G05", since "G03" does not exist in this file
/// //TODO
/// 
/// // Several operands exist, when ommited, the Equality ("eq") operand is implied. 
/// // For example with the GreaterThan (>) operand, we retain 
/// // GPS vehicules above PRN10
/// let mask = Filter::from_str("gt:g01,g02,g03, g05")
///     .unwrap();
///
/// // use AND() to create complex (combined) masks
/// let complex = Filter::from_str("GPS")
///     .unwrap()
///     .and(Filter::from_str("GLO").unwrap());
/// // retrieve GPS + GLONASS vehicules
/// let masked = rinex.apply(mask);
/// ```
#[derive(Debug, Clone)]
pub enum Filter {
    /// Mask filter is used to focus or remove a specific data subset
    Mask(MaskFilter),
    /// Smoothing filter is used to smooth a data subset
    Smoothing(SmoothingFilter),
    /// Decimation filter to reduce sample rate
    Decimation(DecimationFilter),
    /// Interpolation filter to increase sample rate
    Interp(InterpFilter),
}

impl From<MaskFilter> for Filter {
    fn from(mask: MaskFilter) -> Self {
        Self::Mask(mask)
    }
}

impl From<DecimationFilter> for Filter {
    fn from(decim: decim::DecimationFilter) -> Self {
        Self::Decimation(decim)
    }
}

impl From<SmoothingFilter> for Filter {
    fn from(smoothing: SmoothingFilter) -> Self {
        Self::Smoothing(smoothing)
    }
}

impl std::str::FromStr for Filter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.split(":").collect();
        if items.len() < 2 {
            return Err(Error::InvalidDescriptor);
        }
        if items[0].trim().eq("mask") {
            if let Ok(filt) = MaskFilter::from_str(&content[5..].trim()) {
                Ok(Self::Mask(filt))
            } else if let Ok(item) = TargetItem::from_str(&content[5..].trim()) {
                Ok(Self::Mask(MaskFilter {
                    operand: MaskOperand::Equals,
                    item,
                }))
            } else {
                Err(Error::MaskError(content[5..].trim().to_string()))
            }
        } else if items[0].trim().eq("smooth") {
            if let Ok(filt) = SmoothingFilter::from_str(&content[7..].trim()) {
                Ok(Self::Smoothing(filt))
            } else {
                Err(Error::MaskError(content[7..].trim().to_string()))
            }
        } else if items[0].trim().eq("decim") {
            if let Ok(filt) = DecimationFilter::from_str(&content[6..].trim()) {
                Ok(Self::Decimation(filt))
            } else {
                Err(Error::DecimError(content[6..].trim().to_string()))
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
    fn from_str() {
        /*
         * Test MASK FILTER description
         */
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
            assert!(filt.is_ok(), "Filter::from_str failed on \"{}\"", desc);
        }
        /*
         * Test MASK FILTER description
         */
        for desc in vec![
            "mask:10.0",
            "mask:10.0, 13.0",
            "mask:GPS",
            "mask:GPS,GAL",
            "mask:G08, G09, G10",
        ] {
            let filt = Filter::from_str(desc);
            assert!(filt.is_ok(), "Filter::from_str failed on \"{}\"", desc);
        }
        /*
         * Test DECIMATION FILTER description
         */
        for desc in vec!["decim:10", "decim:10 min", "decim:1 hour"] {
            let filt = Filter::from_str(desc);
            assert!(filt.is_ok(), "Filter::from_str failed on \"{}\"", desc);
        }
        /*
         * Test SMOOTHING FILTER description
         */
        for desc in vec![
            "smooth:mov:10 min",
            "smooth:mov:1 hour",
            "smooth:mov:1 hour:l1c",
            "smooth:mov:10 min:clk",
        ] {
            let filt = Filter::from_str(desc);
            assert!(filt.is_ok(), "Filter::from_str failed on \"{}\"", desc);
        }
    }
}
