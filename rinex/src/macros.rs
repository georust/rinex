//! Macros and helpers

/// Creates an [`crate::Sv`] from given string description,
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

#[cfg(feature = "processing")]
#[macro_export]
/// Returns a filter object, from a given description which must be valid
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
    ($x: expr, $y: expr, $z: expr) => {
        GroundPosition::from_ecef_wgs84(($x, $y, $z))
    };
}

/// Builds a [crate::GroundPosition] from geodetic coordinates in ddeg
#[macro_export]
macro_rules! geodetic {
    ($lat: expr, $lon: expr, $alt: expr) => {
        GroundPosition::from_geodetic(($lat, $lon, $alt))
    };
}
