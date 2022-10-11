//! `GNSS` geostationary augmentation systems,
//! mainly used for high precision positioning
use strum_macros::EnumString;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// GNSS Augmentation systems,
/// must be used based on current location
pub enum Augmentation {
    /// Augmentation Unknown
    Unknown,
    /// American augmentation system,
    WAAS,
    /// European augmentation system
    EGNOS,
    /// Japanese augmentation system
    MSAS,
    /// Indian augmentation system
    GAGAN,
    /// Chinese augmentation system
    BDSBAS,
    /// South Korean augmentation system
    KASS,
    /// Russian augmentation system
    SDCM,
    /// South African augmentation system
    ASBAS,
    /// Autralia / NZ augmentation system
    SPAN,
}

impl Default for Augmentation {
    fn default() -> Augmentation {
        Augmentation::Unknown
    }
}

#[cfg(feature = "with-geo")]
use std::str::FromStr;
#[cfg(feature = "with-geo")]
use std::iter::FromIterator;
#[cfg(feature = "with-geo")]
use wkt::{Geometry, Wkt, WktFloat};
#[cfg(feature = "with-geo")]
use geo::{point, Contains, LineString};

#[cfg(feature = "with-geo")]
fn wkt_line_string_to_geo<T> (line_string: &wkt::types::LineString<T>) -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    LineString::from_iter(line_string.0
        .iter()
        .map(|coord| (coord.x, coord.y)))
}

#[cfg(feature = "with-geo")]
fn line_string<T>(name: &str) -> LineString<T>
where 
    T: WktFloat + Default + FromStr,
{
    let mut res = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    res.push("db");
    res.push("SBAS");
    res.push(name);
    let content = std::fs::read_to_string(res)
        .unwrap();
    let wkt = Wkt::from_str(&content)
        .unwrap();
    match wkt.item {
        Geometry::LineString(line) => wkt_line_string_to_geo(&line),
        _ => unreachable!(),
    }
}

#[cfg(feature = "with-geo")]
fn load_database() -> Vec<(Augmentation, geo::Polygon)> {
    let mut db :Vec<(Augmentation, geo::Polygon)> = Vec::new();
    let db_path = env!("CARGO_MANIFEST_DIR")
        .to_owned()
        + "/db/SBAS/";
    let db_path = std::path::PathBuf::from(db_path);
    for entry in std::fs::read_dir(db_path)
        .unwrap()
    {
        let entry = entry
            .unwrap();
        let path = entry.path();
        let fullpath = &path.to_str()
            .unwrap();
        let extension = path.extension()
            .unwrap()
            .to_str()
            .unwrap();
        let name = path.file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        if extension.eq("wkt") {
            let poly = geo::Polygon::<f64>::new(
                line_string(fullpath), // exterior boundaries
                vec![]); // dont care about interior
            if let Ok(sbas) = Augmentation::from_str(&name.to_uppercase()) {
                db.push((sbas, poly))
            }
        }
    }
    db
}

/// SBAS augmentation system selection helper,
/// returns most approriate Augmentation system
/// depending on given location, latitude: in [ddeg]
/// and longitude: in [ddeg]
/* TODO
/// Example:
/// ```
/// use rinex::*;
/// use rinex::constellation::sbas_selection_helper;
/// let paris = (48.808378, 2.382682); // lat, lon [ddeg]
/// let sbas = sbas_selection_helper(paris.0, paris.1);
/// assert_eq!(sbas, Some(Augmentation::EGNOS));
/// let antartica = (-77.490631,  91.435181); // lat, lon [ddeg]
/// let sbas = sbas_selection_helper(antartica.0, antartica.1);
/// assert_eq!(sbas.is_none(), true);
///```
*/
#[cfg(feature = "with-geo")]
pub fn sbas_selection_helper (lat: f64, lon: f64) -> Option<Augmentation> {
    let db = load_database();
    let point : geo::Point<f64> = point!(
        x: lon,
        y: lat,
    );
    for (sbas, area) in db {
        if area.contains(&point) {
            return Some(sbas.clone())
        }
    }
    None
}

#[cfg(feature = "with-geo")]
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    #[cfg(feature = "with-geo")]
    fn test_sbas_selection() {
        // PARIS --> EGNOS
        let sbas = sbas_selection_helper(48.808378, 2.382682);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Augmentation::EGNOS);
        
        // ANTARICA --> NONE
        let sbas = sbas_selection_helper(-77.490631,  91.435181);
        assert_eq!(sbas.is_none(), true);
        
        // LOS ANGELES --> WAAS
        let sbas = sbas_selection_helper(33.981431, -118.193601);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Augmentation::WAAS);
        
        // ARGENTINA --> NONE
        let sbas = sbas_selection_helper(-23.216639, -63.170983);
        assert_eq!(sbas.is_none(), true);

        // NIGER --> ASBAS
        let sbas = sbas_selection_helper(10.714217, 17.087263);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Augmentation::ASBAS);
        
        // South AFRICA --> None
        let sbas = sbas_selection_helper(-32.473320, 21.112770);
        assert_eq!(sbas.is_none(), true);

        // India --> GAGAN
        let sbas = sbas_selection_helper(19.314290, 76.798953);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Augmentation::GAGAN);

        // South Indian Ocean --> None
        let sbas = sbas_selection_helper(-29.349172, 72.773447);
        assert_eq!(sbas.is_none(), true);
    
        // Australia --> SPAN
        let sbas = sbas_selection_helper(-27.579847, 131.334992);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Augmentation::SPAN);
        // NZ --> SPAN
        let sbas = sbas_selection_helper(-45.113525, 169.864842);
        assert_eq!(sbas.is_some(), true);
        assert_eq!(sbas.unwrap(), Augmentation::SPAN);

        // Central China: BDSBAS
        let sbas = sbas_selection_helper(34.462967, 98.172480);
        assert_eq!(sbas, Some(Augmentation::BDSBAS));

        // South Korea: KASS
        let sbas = sbas_selection_helper(37.067846, 128.34);
        assert_eq!(sbas, Some(Augmentation::KASS));
        
        // Japan: MSAS
        let sbas = sbas_selection_helper(36.081095, 138.274859);
        assert_eq!(sbas, Some(Augmentation::MSAS));

        // Russia: SDCM
        let sbas = sbas_selection_helper(60.004390, 89.090326);
        assert_eq!(sbas, Some(Augmentation::SDCM));
    }
}
