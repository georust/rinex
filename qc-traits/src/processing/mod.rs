//! Processing toolkit, including filter designer.
use thiserror::Error;

mod item;
pub use item::{FilterItem, ItemError};

mod mask;
pub use mask::{Error as MaskError, MaskFilter, MaskOperand, Masking};

/// Preprocessing Trait is usually implemented by GNSS data
/// to preprocess prior further analysis.
pub trait Preprocessing: Masking {
    /// Apply [Filter] algorithm on immutable dataset.
    fn filter(&self, filter: &Filter) -> Self
    where
        Self: Sized,
    {
        match filter {
            Filter::Mask(f) => self.mask(f),
        }
    }
    /// Apply [Filter] algorithm on mutable dataset.
    fn filter_mut(&mut self, filter: &Filter) {
        match filter {
            Filter::Mask(f) => self.mask_mut(f),
        }
    }
}

// pub use filters::{
//     Decimate, DecimationFilter, DecimationType, Filter, InterpFilter, InterpMethod, Interpolate,
//     Mask, MaskFilter, MaskOperand, Preprocessing, Smooth, SmoothingFilter, SmoothingType,
// };

//pub use averaging::Averager;
//pub use derivative::Derivative;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid filter")]
    InvalidFilter,
    #[error("unknown filter type \"{0}\"")]
    UnknownFilterType(String),
    #[error("invalid mask filter")]
    MaskFilterParsing(#[from] MaskError),
    // #[error("invalid decimation filter")]
    // DecimationFilterParsing(#[from] decim::Error),
    // #[error("invalid smoothing filter")]
    // SmoothingFilterParsing(#[from] smoothing::Error),
    #[error("invalid filter item")]
    FilterItemError(#[from] ItemError),
}

/// Preprocessing filters, to preprocess RINEX data prior further analysis.
/// Filters can apply either on entire RINEX or subsets.
/// Refer to [TargetItem] definition to understand which data subsets exist.  
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    /// Mask filter, to focus on specific data subsets
    Mask(MaskFilter),
    // /// Smoothing filters, used to smooth a data subset
    // Smoothing(SmoothingFilter),
    // /// Decimation filter, filters to reduce sample rate
    // Decimation(DecimationFilter),
    // /// Interpolation filter is work in progress and cannot be used at the moment
    // Interp(InterpFilter),
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
        }
    }
}

// impl From<DecimationFilter> for Filter {
//     fn from(decim: decim::DecimationFilter) -> Self {
//         Self::Decimation(decim)
//     }
// }
//
// impl From<SmoothingFilter> for Filter {
//     fn from(smoothing: SmoothingFilter) -> Self {
//         Self::Smoothing(smoothing)
//     }
// }

impl std::str::FromStr for Filter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.split(':').collect();

        let identifier = items[0].trim();
        //if identifier.eq("decim") {
        //    let offset = 6; //"decim:"
        //    Ok(Self::Decimation(DecimationFilter::from_str(
        //        content[offset..].trim(),
        //    )?))
        //} else if identifier.eq("smooth") {
        //    let offset = 7; //"smooth:"
        //    Ok(Self::Smoothing(SmoothingFilter::from_str(
        //        content[offset..].trim(),
        //    )?))
        //} else if identifier.eq("interp") {
        //    todo!("InterpolationFilter::from_str()");
        //} else
        if identifier.eq("mask") {
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

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn from_str() {
        /*
         * MASK FILTER description
         */
        for descriptor in [
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
        for desc in [
            "decim:10",
            "decim:10 min",
            "decim:1 hour",
            "decim:10 min:l1c",
            "decim:1 hour:L1C,L2C,L3C",
        ] {
            let filt = Filter::from_str(desc);
            assert!(filt.is_ok(), "Filter::from_str failed on \"{}\"", desc);
        }
        /*
         * SMOOTHING FILTER description
         */
        for desc in [
            "smooth:mov:10 min",
            "smooth:mov:1 hour",
            "smooth:mov:1 hour:l1c",
            "smooth:mov:10 min:clk",
            "smooth:hatch",
            "smooth:hatch:l1c",
        ] {
            let filt = Filter::from_str(desc);
            assert!(filt.is_ok(), "Filter::from_str failed on \"{}\"", desc);
        }
    }
}
