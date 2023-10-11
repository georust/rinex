//! Geostationary augmentation systems
use crate::prelude::Constellation;

//#[cfg(feature = "serde")]
//use serde::{Deserialize, Serialize};

#[cfg(feature = "sbas")]
use geo::{point, Contains, LineString};
#[cfg(feature = "sbas")]
use std::iter::FromIterator;
#[cfg(feature = "sbas")]
use std::str::FromStr;
#[cfg(feature = "sbas")]
use wkt::{Geometry, Wkt, WktFloat};

#[cfg(feature = "sbas")]
fn wkt_line_string_to_geo<T>(line_string: &wkt::types::LineString<T>) -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    LineString::from_iter(line_string.0.iter().map(|coord| (coord.x, coord.y)))
}

#[cfg(feature = "sbas")]
fn line_string<T>(name: &str) -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    let mut res = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    res.push("data");
    res.push(name);
    let content = std::fs::read_to_string(res).unwrap();
    let wkt = Wkt::from_str(&content).unwrap();
    match wkt.item {
        Geometry::LineString(line) => wkt_line_string_to_geo(&line),
        _ => unreachable!(),
    }
}

#[cfg(feature = "sbas")]
fn load_database() -> Vec<(Constellation, geo::Polygon)> {
    let mut db: Vec<(Constellation, geo::Polygon)> = Vec::new();
    let db_path = env!("CARGO_MANIFEST_DIR").to_owned() + "/data/";
    let db_path = std::path::PathBuf::from(db_path);
    for entry in std::fs::read_dir(db_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let fullpath = &path.to_str().unwrap();
        let extension = path.extension().unwrap().to_str().unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();
        if extension.eq("wkt") {
            let poly = geo::Polygon::<f64>::new(
                line_string(fullpath), // exterior boundaries
                vec![],
            ); // dont care about interior
            if let Ok(sbas) = Constellation::from_str(&name.to_uppercase()) {
                db.push((sbas, poly))
            }
        }
    }
    db
}

#[cfg(feature = "sbas")]
#[cfg_attr(docrs, doc(cfg(feature = "sbas")))]
/// Select an augmentation system conveniently, based on given location
/// in decimal degrees
/// ```
/// use rinex::prelude::*;
/// use rinex::constellation::sbas_selection_helper;
///
/// let paris = (48.808378, 2.382682); // lat, lon [ddeg]
/// let sbas = sbas_selection_helper(paris.0, paris.1);
/// assert_eq!(sbas, Some(Constellation::EGNOS));
///
/// let antartica = (-77.490631,  91.435181); // lat, lon [ddeg]
/// let sbas = sbas_selection_helper(antartica.0, antartica.1);
/// assert_eq!(sbas.is_none(), true);
///```
pub fn sbas_selection_helper(lat: f64, lon: f64) -> Option<Constellation> {
    let db = load_database();
    let point: geo::Point<f64> = point!(x: lon, y: lat,);
    for (sbas, area) in db {
        if area.contains(&point) {
            return Some(sbas.clone());
        }
    }
    None
}

#[cfg(feature = "sbas")]
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    #[cfg(feature = "sbas")]
    fn sbas_helper() {
        // PARIS --> EGNOS
        let sbas = sbas_selection_helper(48.808378, 2.382682);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Constellation::EGNOS);

        // ANTARICA --> NONE
        let sbas = sbas_selection_helper(-77.490631, 91.435181);
        assert_eq!(sbas.is_none(), true);

        // LOS ANGELES --> WAAS
        let sbas = sbas_selection_helper(33.981431, -118.193601);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Constellation::WAAS);

        // ARGENTINA --> NONE
        let sbas = sbas_selection_helper(-23.216639, -63.170983);
        assert_eq!(sbas.is_none(), true);

        // NIGER --> ASBAS
        let sbas = sbas_selection_helper(10.714217, 17.087263);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Constellation::ASBAS);

        // South AFRICA --> None
        let sbas = sbas_selection_helper(-32.473320, 21.112770);
        assert_eq!(sbas.is_none(), true);

        // India --> GAGAN
        let sbas = sbas_selection_helper(19.314290, 76.798953);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Constellation::GAGAN);

        // South Indian Ocean --> None
        let sbas = sbas_selection_helper(-29.349172, 72.773447);
        assert_eq!(sbas.is_none(), true);

        // Australia --> SPAN
        let sbas = sbas_selection_helper(-27.579847, 131.334992);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Constellation::SPAN);
        // NZ --> SPAN
        let sbas = sbas_selection_helper(-45.113525, 169.864842);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Constellation::SPAN);

        // Central China: BDSBAS
        let sbas = sbas_selection_helper(34.462967, 98.172480);
        assert_eq!(sbas, Some(Constellation::BDSBAS));

        // South Korea: KASS
        let sbas = sbas_selection_helper(37.067846, 128.34);
        assert_eq!(sbas, Some(Constellation::KASS));

        // Japan: MSAS
        let sbas = sbas_selection_helper(36.081095, 138.274859);
        assert_eq!(sbas, Some(Constellation::MSAS));

        // Russia: SDCM
        let sbas = sbas_selection_helper(60.004390, 89.090326);
        assert_eq!(sbas, Some(Constellation::SDCM));
    }
}
