use crate::processing::{FilterItem, ItemError};
use hifitime::Duration;
use thiserror::Error;

/// Decimation filter parsing error
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid decimated item")]
    InvalidDecimItem(#[from] ItemError),
    #[error("failed to parse decimation attribute \"{0}\"")]
    AttributeParsingError(String),
}

/// Type of decimation filter
#[derive(Clone, Debug, PartialEq)]
pub enum DecimationFilterType {
    /// Simple modulo decimation
    Modulo(u32),
    /// Duration decimation
    Interval(Duration),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecimationFilter {
    /// Type of decimation filter
    pub filter: DecimationFilterType,
    /// Optional decimated item.
    /// When item is None, all data is to be decimated.
    /// When item is specified, only that subset is to be decimated.
    pub item: Option<FilterItem>,
}

/// The [Decimate] trait is implemented to reduce data rate prior analysis.
pub trait Decimate {
    /// Immutable decimation
    fn decimate(&self, f: &DecimationFilter) -> Self;
    /// Mutable decimation
    fn decimate_mut(&mut self, f: &DecimationFilter);
}

impl std::str::FromStr for DecimationFilter {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = content.trim().split(':').collect();
        if let Ok(dt) = Duration::from_str(items[0].trim()) {
            Ok(Self {
                item: {
                    if items.len() > 1 {
                        let item = FilterItem::from_str(items[1].trim())?;
                        Some(item)
                    } else {
                        None // no subset description
                    }
                },
                filter: DecimationFilterType::Interval(dt),
            })
        } else if let Ok(r) = items[0].trim().parse::<u32>() {
            Ok(Self {
                item: {
                    if items.len() > 1 {
                        let item = FilterItem::from_str(items[1].trim())?;
                        Some(item)
                    } else {
                        None
                    }
                },
                filter: DecimationFilterType::Modulo(r),
            })
        } else {
            Err(Error::AttributeParsingError(items[0].to_string()))
        }
    }
}
