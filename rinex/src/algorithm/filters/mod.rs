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
    #[error("invalid mask filter")]
    MaskFilterParsingError(#[from] mask::Error),
    #[error("invalid decimation filter")]
    DecimationFilterParsingError(#[from] decim::Error),
    #[error("invalid smoothing filter")]
    SmoothingFilterParsingError(#[from] smoothing::Error),
    #[error("invalid filter target")]
    TargetItemError(#[from] super::target::Error),
    #[error("failed to apply filter")]
    FilterError,
}

/// Preprocessing filters, to preprocess RINEX data prior further analysis.
/// Filters can apply either on entire RINEX or subsets.
/// Refer to [TargetItem] definition to understand which data subsets exist.  
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    /// Mask filter, to focus on specific data subsets
    Mask(MaskFilter),
    /// Smoothing filters, used to smooth a data subset
    Smoothing(SmoothingFilter),
    /// Decimation filter, filters to reduce sample rate
    Decimation(DecimationFilter),
    /// Interpolation filter is work in progress and cannot be used at the moment
    Interp(InterpFilter),
}

impl From<MaskFilter> for Filter {
    fn from(mask: MaskFilter) -> Self {
        Self::Mask(mask)
    }
}

impl std::ops::Not for Filter {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::Mask(f) => Self::Mask(!f),
            f => f.clone(), // impossible
        }
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

        let identifier = items[0].trim();
        if identifier.eq("decim") {
            let offset = 6; //"decim:"
            Ok(Self::Decimation(DecimationFilter::from_str(
                content[offset..].trim(),
            )?))
        } else if identifier.eq("smooth") {
            let offset = 7; //"smooth:"
            Ok(Self::Smoothing(SmoothingFilter::from_str(
                content[offset..].trim(),
            )?))
        } else if identifier.eq("interp") {
            todo!("InterpolationFilter::from_str()");
        } else if identifier.eq("mask") {
            let offset = 5; //"mask:"
            Ok(Self::Mask(MaskFilter::from_str(content[offset..].trim())?))
        } else {
            // assume Mask (omitted identifier)
            if let Ok(f) = MaskFilter::from_str(content.trim()) {
                Ok(Self::Mask(f))
            } else {
                Err(Error::UnknownFilterType(content.to_string()))
            }
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
         * MASK FILTER description
         */
        for descriptor in vec![
            "GPS",
            "=GPS",
            " != GPS",
            "G08, G09, G10",
            "=G08, G09, G10",
            "!= GPS, GAL",
            ">G08, G09",
            "iode",
            "iode,gps",
            "iode,crs,gps",
            "iode,crs",
            ">2020-01-14T00:31:55 UTC",
        ] {
            assert!(
                Filter::from_str(descriptor).is_ok(),
                "Filter::from_str failed on \"{}\"",
                descriptor
            );
        }
        /*
         * DECIMATION FILTER description
         */
        for desc in vec!["decim:10", "decim:10 min", "decim:1 hour"] {
            let filt = Filter::from_str(desc);
            assert!(filt.is_ok(), "Filter::from_str failed on \"{}\"", desc);
        }
        /*
         * SMOOTHING FILTER description
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
