use crate::processing::{FilterItem, ItemError, Error};

/// Supported Smoothing Filters
pub enum SmoothingFilterType {
    /// Special Hatch smoothing filter
    Hatch,
}

impl std::str::FromStr for SmoothingFilterType {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let trimed = content.trim().lowercase();
        match trimed {
            "hatch" => Ok(Self::Hatch),
            _ => {
                Err(Error::InvalidFilter),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SmoothingFilter {
    /// [SmoothingFilterType]
    pub filter: SmoothingFilterType,
    /// Optional smoothed item.
    /// All dataset to be smoothed when omitted.
    pub item: Option<FilterItem>,
}

/// Applies [SmoothingFilter] algorithm.
pub trait Smoothing {
    fn smoothing(&self, f: &SmoothingFilter) -> Self;
    fn smoothing_mut(&mut self, f: &SmoothingFilter);
}
