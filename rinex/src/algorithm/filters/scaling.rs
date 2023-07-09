use crate::processing::TargetItem;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown scaling target")]
    TargetError(#[from] crate::algorithm::target::Error),
    #[error("failed to parse scaling value from \"{0}\"")]
    ScalingParsingError(String),
    #[error("unknown scaling type \"{0}\"")]
    UnknownScalingType(String),
    #[error("failed to parse bins value in \"{0}\"")]
    ScalingBinParsing(String),
    #[error("failed to parse offset value in \"{0}\"")]
    ScalingOffsetParsing(String),
}

/// Scaling Filters type
#[derive(Clone, Debug, PartialEq)]
pub enum ScalingType {
    /// Apply a static offset: y_k = x_k + a
    Offset(f64),
    /// Rescale dataset so all data terms x_k fit in a
    /// X = {x_1, ..., x_usize} ensemble
    Rescale(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScalingFilter {
    /// scaling type
    pub stype: ScalingType,
    /// optionnal data subset to scale.
    /// When undefined, the filter applies to entire dataset
    pub target: Option<TargetItem>,
}

pub trait Scale {
    /// Apply given scaling filter
    fn scale(&self, scaling: ScalingFilter) -> Self;
    /// Apply given scaling filter in place
    fn scale_mut(&mut self, scaling: ScalingFilter);
    /// Offset dataset or subset by a static value
    /// y_k = x_k + b
    fn offset(&self, b: f64) -> Self;
    /// Offset dataset or subset by a static value
    /// y_k = x_k + b
    fn offset_mut(&mut self, b: f64);
    /// Rescale dataset or subset
    fn rescale(&self, bins: usize) -> Self;
    /// Rescale dataset or subset  
    fn rescale_mut(&mut self, bins: usize);
}

impl std::str::FromStr for ScalingFilter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.trim().split(":").collect();
        if items[0].eq("rescale") {
            if let Ok(bins) = u32::from_str_radix(items[1].trim(), 10) {
                Ok(Self {
                    stype: ScalingType::Rescale(bins as usize),
                    target: {
                        if items.len() > 2 {
                            let target = TargetItem::from_str(items[2].trim())?;
                            Some(target)
                        } else {
                            None
                        }
                    },
                })
            } else {
                Err(Error::ScalingBinParsing(content.to_string()))
            }
        } else if items[0].eq("offset") {
            if let Ok(f) = f64::from_str(items[1].trim()) {
                Ok(Self {
                    stype: ScalingType::Offset(f),
                    target: {
                        if items.len() > 2 {
                            let target = TargetItem::from_str(items[2].trim())?;
                            Some(target)
                        } else {
                            None
                        }
                    },
                })
            } else {
                Err(Error::ScalingOffsetParsing(content.to_string()))
            }
        } else {
            Err(Error::UnknownScalingType(items[0].to_string()))
        }
    }
}
