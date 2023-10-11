/// Creates a [SV] from given string description.
#[macro_export]
macro_rules! sv {
    ($desc: expr) => {
        SV::from_str($desc).unwrap()
    };
}

/// Crates a [Constellation] from given string description.
#[macro_export]
macro_rules! gnss {
    ($desc: expr) => {
        Constellation::from_str($desc).unwrap()
    };
}
