//mod averaging;
mod derivative;
mod filters;

pub use filters::{
    Decimate, DecimationFilter, DecimationType, Filter, InterpFilter, InterpMethod, Interpolate,
    Mask, MaskFilter, MaskOperand, Preprocessing, Smooth, SmoothingFilter, SmoothingType,
};

//pub use averaging::Averager;
pub use derivative::Derivative;
