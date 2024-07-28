//! Macros and helpers

/// Creates an [crate::Observable] from given string
/// description, which must be valid.
#[macro_export]
macro_rules! observable {
    ($desc: expr) => {
        Observable::from_str($desc).unwrap()
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
