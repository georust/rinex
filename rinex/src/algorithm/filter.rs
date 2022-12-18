use super::MaskFilter;
use crate::prelude::Epoch;
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum FilterItem {
    EpochFilter(Epoch),
    ElevationFilter(f64),
    //ObservableItem((Observable, f64)),
    //OrbitItem((Orbit, f64)),
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum FilterItemError {
    #[error("unrecognized filter item")]
    UnrecognizedFilterItem,
}

impl std::str::FromStr for FilterItem {
    type Err = FilterItemError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let c = content.trim();
        if let Ok(epoch) = Epoch::from_str(c) {
            Ok(Self::EpochFilter(epoch))
        } else if let Ok(f) = f64::from_str(c) {
            Ok(Self::ElevationFilter(f))
        } else {
            Err(FilterItemError::UnrecognizedFilterItem)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FilterType<I> {
    Mask(MaskFilter<I>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum FilterMode {
    Discard,
    Retain,
}

impl FilterType<FilterItem> {
    pub fn as_mask(&self) -> Option<&MaskFilter<FilterItem>> {
        match self {
            Self::Mask(f) => Some(f),
            _ => None,
        }
    }
}

impl From<MaskFilter<FilterItem>> for FilterType<FilterItem> {
    fn from(mask: MaskFilter<FilterItem>) -> Self {
        Self::Mask(mask)
    }
}

pub trait Filter {
    fn apply(&self, filt: FilterType<FilterItem>, mode: FilterMode) -> Self;
    fn apply_mut(&mut self, filt: FilterType<FilterItem>, mode: FilterMode);
}
