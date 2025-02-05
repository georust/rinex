//! Processing toolkit, including filter designer.
use std::str::FromStr;
use thiserror::Error;

mod item;
pub use item::{FilterItem, ItemError};

mod mask;
pub use mask::{Error as MaskError, MaskFilter, MaskOperand, Masking};

mod decim;
pub use decim::{Decimate, DecimationFilter, DecimationFilterType, Error as DecimationError};

mod split;
pub use split::Split;

/// Preprocessing Trait is usually implemented by GNSS data
/// to preprocess prior further analysis.
pub trait Preprocessing: Masking + Decimate + Split {
    /// Apply [Filter] algorithm on immutable dataset.
    fn filter(&self, filter: &Filter) -> Self
    where
        Self: Sized,
    {
        match filter {
            Filter::Mask(f) => self.mask(f),
            Filter::Decimation(f) => self.decimate(f),
        }
    }
    /// Apply [Filter] algorithm on mutable dataset.
    fn filter_mut(&mut self, filter: &Filter) {
        match filter {
            Filter::Mask(f) => self.mask_mut(f),
            Filter::Decimation(f) => self.decimate_mut(f),
        }
    }
}

/// Repair
#[derive(Debug, Copy, Clone)]
pub enum Repair {
    /// Repairs zero phase range and decoded range values,
    /// that are physically incorrect and most likely generated
    /// by incorrect GNSS receiver behavior.
    Zero,
}

pub trait RepairTrait {
    fn repair(&self, r: Repair) -> Self;
    fn repair_mut(&mut self, r: Repair);
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid filter")]
    InvalidFilter,
    #[error("unknown filter type \"{0}\"")]
    UnknownFilterType(String),
    #[error("invalid mask filter")]
    MaskFilterParsing(#[from] MaskError),
    #[error("invalid filter item")]
    FilterItemError(#[from] ItemError),
    #[error("invalid decimation filter")]
    DecimationFilterParsing(#[from] DecimationError),
}

/// Preprocessing filters, to preprocess RINEX data prior further analysis.
/// Filters can apply either on entire RINEX or subsets.
/// Refer to [TargetItem] definition to understand which data subsets exist.  
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    /// Mask filter, to focus on specific data subsets
    Mask(MaskFilter),
    /// Decimation filter, filters to reduce sample rate
    Decimation(DecimationFilter),
    // /// Interpolation filter is work in progress and cannot be used at the moment
    // Interp(InterpFilter),
}

impl Filter {
    /// Builds new [MaskFilter] from given specs
    pub fn mask(operand: MaskOperand, item: FilterItem) -> Self {
        Self::Mask(MaskFilter { operand, item })
    }
    /// Builds new [MaskFilter] with Equals operand
    /// from following [FilterItem] description
    pub fn equals(item: &str) -> Result<Self, ItemError> {
        let item = FilterItem::from_str(item)?;
        Ok(Self::mask(MaskOperand::Equals, item))
    }
    /// Builds new [MaskFilter] with !Equals operand
    /// from following [FilterItem] description
    pub fn not_equals(item: &str) -> Result<Self, ItemError> {
        let item = FilterItem::from_str(item)?;
        Ok(Self::mask(MaskOperand::NotEquals, item))
    }
    /// Builds new [MaskFilter] with GreaterThan operand
    /// from following [FilterItem] description
    pub fn greater_than(item: &str) -> Result<Self, ItemError> {
        let item = FilterItem::from_str(item)?;
        Ok(Self::mask(MaskOperand::GreaterThan, item))
    }
    /// Builds new [MaskFilter] with GreaterEquals operand
    /// from following [FilterItem] description
    pub fn greater_equals(item: &str) -> Result<Self, ItemError> {
        let item = FilterItem::from_str(item)?;
        Ok(Self::mask(MaskOperand::GreaterEquals, item))
    }
    /// Builds new [MaskFilter] with LowerEquals operand
    /// from following [FilterItem] description
    pub fn lower_equals(item: &str) -> Result<Self, ItemError> {
        let item = FilterItem::from_str(item)?;
        Ok(Self::mask(MaskOperand::LowerEquals, item))
    }
    /// Builds new [MaskFilter] with LowerThan operand
    /// from following [FilterItem] description
    pub fn lower_than(item: &str) -> Result<Self, ItemError> {
        let item = FilterItem::from_str(item)?;
        Ok(Self::mask(MaskOperand::LowerThan, item))
    }
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
            _ => self.clone(), // does not apply
        }
    }
}

impl From<DecimationFilter> for Filter {
    fn from(decim: decim::DecimationFilter) -> Self {
        Self::Decimation(decim)
    }
}

impl std::str::FromStr for Filter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.split(':').collect();

        let identifier = items[0].trim();
        if identifier.eq("decim") {
            let offset = 6; //"decim:"
            Ok(Self::Decimation(DecimationFilter::from_str(
                content[offset..].trim(),
            )?))
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
