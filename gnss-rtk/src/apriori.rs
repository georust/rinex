use crate::Vector3D;
use map_3d::{ecef2geodetic, geodetic2ecef, Ellipsoid};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AprioriPosition {
    pub(crate) ecef: Vector3D,
    pub(crate) geodetic: Vector3D,
}

impl AprioriPosition {
    /// Builds Self from ECEF position [m]
    pub fn from_ecef(ecef: Vector3D) -> Self {
        let (x, y, z) = (ecef.x, ecef.y, ecef.z);
        Self {
            ecef,
            geodetic: ecef2geodetic(x, y, z, Ellipsoid::WGS84).into(),
        }
    }
    /// Builds Self from Geodetic coordinates: 
    /// latitude [ddeg], longitude [ddeg] and altitude above sea [m].
    pub fn from_geo(geodetic: Vector3D) -> Self {
        let (lat, lon, alt) = (geodetic.x, geodetic.y, geodetic.z);
        Self {
            ecef: geodetic2ecef(lat, lon, alt, Ellipsoid::WGS84).into(),
            geodetic,
        }
    }
}

