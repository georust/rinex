use map_3d::{deg2rad, ecef2geodetic, geodetic2ecef, rad2deg, Ellipsoid};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Copy, Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroundPosition(f64, f64, f64);

impl GroundPosition {
    pub fn from_ecef_wgs84(pos: (f64, f64, f64)) -> Self {
        Self(pos.0, pos.1, pos.2)
    }
    pub fn from_geodetic(pos: (f64, f64, f64)) -> Self {
        let (x, y, z) = pos;
        let (x, y, z) = geodetic2ecef(deg2rad(x), deg2rad(y), deg2rad(z), Ellipsoid::WGS84);
        Self(x, y, z)
    }
    pub fn to_ecef_wgs84(&self) -> (f64, f64, f64) {
        (self.0, self.1, self.2)
    }
    pub fn to_geodetic(&self) -> (f64, f64, f64) {
        let (x, y, z) = (self.0, self.1, self.2);
        let (lat, lon, alt) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
        (rad2deg(lat), rad2deg(lon), alt)
    }
}

impl std::fmt::Display for GroundPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "WGS84 ({}m {}m {}m)", self.0, self.1, self.2)
    }
}

#[cfg(feature = "qc")]
use crate::quality::HtmlReport;

#[cfg(feature = "qc")]
use horrorshow::RenderBox;

#[cfg(feature = "qc")]
impl HtmlReport for GroundPosition {
    fn to_html(&self) -> String {
        todo!()
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        let ecef = (self.0, self.1, self.2);
        let geo = self.to_geodetic();
        box_html! {
            tr {
                th {
                    : "ECEF (WGS84)"
                }
                td {
                    : format!("X: {:.6} m", ecef.0)
                }
                td {
                    : format!("Y: {:.6} m", ecef.1)
                }
                td {
                    : format!("Z: {:.6} m", ecef.2)
                }
            }
            tr {
                th {
                    : "GEO"
                }
                td {
                    : format!("Lat.: {:.6}°", geo.0)
                }
                td {
                    : format!("Lon.: {:.6}°", geo.1)
                }
                td {
                    : format!("Alt.: {:.6} m", geo.2)
                }
            }
        }
    }
}
