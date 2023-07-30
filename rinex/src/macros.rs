//! Macros and helpers

/// Creates an [crate::Sv] from given string description,
/// which must be valid.
#[macro_export]
macro_rules! sv {
    ($desc: expr) => {
        Sv::from_str($desc).unwrap()
    };
}

/// Creates a [crate::Constellation] from given string
/// description, which must be valid.
#[macro_export]
macro_rules! gnss {
    ($desc: expr) => {
        Constellation::from_str($desc).unwrap()
    };
}

/// Creates an [crate::Observable] from given string
/// description, which must be valid.
#[macro_export]
macro_rules! observable {
    ($desc: expr) => {
        Observable::from_str($desc).unwrap()
    };
}

/// Returns a filter object, from a given description which must be valid
#[macro_export]
macro_rules! filter {
    ($desc: expr) => {
        Filter::from_str($desc).unwrap()
    };
}

/// Returns `true` if given `Rinex` line is a comment
#[macro_export]
macro_rules! is_comment {
    ($line: expr) => {
        $line.trim_end().ends_with("COMMENT")
    };
}

/// Builds a [crate::GroundPosition] in WGS84
#[macro_export]
macro_rules! wgs84 {
    ($desc: expr) => {
        GroundPosition::from_ecef_wgs84(($desc.0, $desc.1, $desc.2))
    };
}

/// Builds a [crate::GroundPosition] from geodetic coordinates in ddeg
#[macro_export]
macro_rules! geodetic {
    ($desc: expr) => {
        GroundPosition::from_geodetic(($desc.0, $desc.1, $desc.2))
    };
}
