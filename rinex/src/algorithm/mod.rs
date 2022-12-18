mod sampling;
pub use sampling::Decimation;

mod filter;
pub use filter::{Filter, MaskFilter, FilterOperand, FilterItem, FilterParsingError};

//pub mod averaging;
