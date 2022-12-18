mod sampling;
pub use sampling::Decimation;

mod filter;
pub use filter::{Filter, FilterItem, FilterItemError, FilterMode, FilterType};

mod mask;
pub use mask::{MaskFilter, MaskOperand};

/*pub mod averaging;

pub enum Filter<Item> {
    Mask(MaskFilter<Item>),
}
*/
