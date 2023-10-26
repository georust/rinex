use crate::Vector3D;
use map_3d::{ecef2geodetic, geodetic2ecef};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AprioriPosition {
    pub(crate) ecef: Vector3D,
    pub(crate) geodetic: Vector3D,
}

impl AprioriPosition {
    /// Builds Self from ECEF position [m]
    pub fn from_ecef(ecef: Vector3D) -> Self {
        Self {
            ecef,
            geodetic: ecef2geodetic(ecef),
        }
    }
    /// Builds Self from Geodetic coordinates: 
    /// latitude [ddeg], longitude [ddeg] and altitude above sea [m].
    pub fn from_geo(geodetic: Vector3D) -> Self {
        Self {
            geodetic,
            position: geodetic2ecef(geodetic),
        }
    }
}

