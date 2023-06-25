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
    /// Rescale dataset with: y_k = x_k * a + b
    Scale((f64, f64)),
    /// Remap dataset in terms of fraction of maximal value.
    /// Maximal value for a dataset is defined as max|x_k|
    /// for all Epoch k.
    Remap(usize),
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
    /// Offset dataset or subset by a static value
    /// y_k = x_k + b
    fn offset(&self, b: f64) -> Self;
    /// Offset dataset or subset by a static value
    /// y_k = x_k + b
    fn offset_mut(&mut self, b: f64);
    /// Scale dataset or subset values to
    /// y_k = x_k * a + b
    fn scale(&self, a: f64, b: f64) -> Self;
    /// Scale dataset or subset values to
    /// y_k = x_k * a + b
    fn scale_mut(&mut self, a: f64, b: f64);
    /// Rescale dataset or subset
    fn remap(&self, bins: usize) -> Self;
    /// Rescale dataset or subset  
    fn remap_mut(&mut self, bins: usize);
}

impl std::str::FromStr for ScalingFilter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.trim().split(":").collect();
        if items[0].eq("remap") {
            if let Ok(bins) = u32::from_str_radix(items[1].trim(), 10) {
                Ok(Self {
                    stype: ScalingType::Remap(bins as usize),
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
