//! Macros and helpers 

/// Creates an [sv::Sv] from given string description,
/// which must be valid.
macro_rules! sv {
    ($desc: expr) => {
        Sv::from_str($desc).unwrap()
    };
}

/// Creates a [constellation::Constellation] from given string
/// description, which must be valid.
macro_rules! gnss {
    ($desc: expr) => {
        Constellation::from_str($desc).unwrap()
    };
}

/// Creates an [observable::Observable] from given string
/// description, which must be valid.
macro_rules! observable {
    ($desc: expr) => {
        Observable::from_str($desc).unwrap()
    };
}

/// Returns a filter object, from a given description which must be valid
macro_rules! filter {
    ($desc: expr) => {
        Filter::from_str($desc).unwrap()
    };
}
