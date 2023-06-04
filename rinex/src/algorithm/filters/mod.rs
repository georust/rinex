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

/// Preprocessing filters, to preprocess RINEX data prior further analysis.
/// Filters can apply either on entire RINEX or subsets.
/// Refer to [TargetItem] definition to understand which data subsets exist.  
/// ```
#[derive(Debug, Clone)]
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
        if let Ok(f) = MaskFilter::from_str(content.trim()) {
            Ok(Self::Mask(f))
        } else if let Ok(f) = DecimationFilter::from_str(content.trim()) {
            Ok(Self::Decimation(f))
        } else if let Ok(f) = SmoothingFilter::from_str(content.trim()) {
            Ok(Self::Smoothing(f))
        } else {
            Err(Error::UnknownFilterType(content.to_string()))
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
        for (verbal_desc, math_desc) in vec![
            ("eq:GPS", "=:GPS"),
            ("neq: GPS", "!=:GPS"),
            ("eq:G08, G09, G10", "=:G08,G09,G10"),
            ("neq:GPS, GAL", "!=:GPS, GAL"),
            ("gt: G08, G09", ">:G08, G09"),
            ("eq:GPS", "!=:GPS"),
            ("eq:GPS, GAL", "=:GPS"),
            ("eq:G08, G09", "=:G08, G09"),
        ] {
            let verbal_filt = Filter::from_str(verbal_desc);
            assert!(verbal_filt.is_ok(), "Filter::from_str failed on \"{}\"", verbal_desc);

            let math_filt = Filter::from_str(math_desc);
            assert!(math_filt.is_ok(), "Filter::from_str failed on \"{}\"", math_desc);
        }
        /*
         * MASK FILTER description (omitted operand)
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
