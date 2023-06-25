mod combination;
mod dcb;
mod filters;
mod ionospheric;
mod processing;
mod target;

pub use combination::{Combination, Combine};
pub use dcb::{Dcb, Mp};
pub use ionospheric::IonoDelayDetector;
pub use target::TargetItem;

pub use processing::Processing;

pub use filters::{
    Decimate, DecimationFilter, DecimationType, Filter, InterpFilter, InterpMethod, Interpolate,
    Mask, MaskFilter, MaskOperand, Preprocessing, Scale, ScalingFilter, ScalingType, Smooth,
    SmoothingFilter, SmoothingType,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatisticalOpsParsingError {
    #[error("unknown operation \"{0}\"")]
    UnknownOperation(String),
}

/// Known statistical quantities or operations
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StatisticalOps {
    Max,
    Min,
    MaxAbs,
    MinAbs,
    Variance,
    StdDev,
    Mean,
    QuadMean,
    HarmMean,
    GeoMean,
}

impl std::str::FromStr for StatisticalOps {
    type Err = StatisticalOpsParsingError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = content.trim().to_lowercase();
        match content.as_str() {
            "max" => Ok(Self::Max),
            "min" => Ok(Self::Min),
            "|max|" => Ok(Self::MaxAbs),
            "|min|" => Ok(Self::MinAbs),
            "var" => Ok(Self::Variance),
            "stddev" => Ok(Self::StdDev),
            "mean" => Ok(Self::Mean),
            "qmean" => Ok(Self::QuadMean),
            "hmean" => Ok(Self::HarmMean),
            "gmean" => Ok(Self::GeoMean),
            _ => Err(StatisticalOpsParsingError::UnknownOperation(content)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn from_str() {
        for (desc, expected) in vec![
            ("max", StatisticalOps::Max),
            ("|max|", StatisticalOps::MaxAbs),
            ("min", StatisticalOps::Min),
            ("|min|", StatisticalOps::MinAbs),
            ("var", StatisticalOps::Variance),
            ("stddev", StatisticalOps::StdDev),
            ("mean", StatisticalOps::Mean),
            ("qmean", StatisticalOps::QuadMean),
            ("hmean", StatisticalOps::HarmMean),
            ("gmean", StatisticalOps::GeoMean),
        ] {
            let value = StatisticalOps::from_str(desc);
            assert!(
                value.is_ok(),
                "failed to parse statistical ops \"{}\"",
                desc
            );
            assert_eq!(value.unwrap(), expected);
        }
    }
}
