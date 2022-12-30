use map_3d::Ellipsoid;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[derive(Copy, Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroundPosition(f64,f64,f64);

impl GroundPosition {
	pub fn from_ecef_wgs84(pos: (f64,f64,f64)) -> Self {
		Self(pos.0, pos.1, pos.2)
	}
	pub fn from_geodetic(pos: (f64,f64,f64)) -> Self {
		let (x, y, z) = pos;
		let (x, y, z) = map_3d::geodetic2ecef(x, y, z, Ellipsoid::WGS84);
		Self(x, y, z)
	}
	pub fn to_ecef_wgs84(&self) -> (f64, f64, f64) {
		(self.0, self.1, self.2)
	}
	pub fn to_geodetic(&self) -> (f64,f64,f64) {
		let (x, y, z) = (self.0, self.1, self.2);
		map_3d::ecef2geodetic(x, y, z, Ellipsoid::WGS84)
	}
}

use horrorshow::RenderBox;
use crate::quality::HtmlReport;

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
					: format!("X: {:.6}", ecef.0)
				}
				td {
					: format!("Y: {:.6}", ecef.1)
				}
				td {
					: format!("Z: {:.6}", ecef.2)
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
