//! `GNSS` constellations & associated methods
use strum_macros::EnumString;

#[cfg(feature = "with-serde")]
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(EnumString)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
/// GNSS Augmentation systems,
/// must be used based on current location
pub enum Augmentation {
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
        Augmentation::WAAS
    }
}

#[cfg(feature = "with-geo")]
use std::str::FromStr;
#[cfg(feature = "with-geo")]
use std::iter::FromIterator;
#[cfg(feature = "with-geo")]
use wkt::{Geometry, Wkt, WktFloat};
#[cfg(feature = "with-geo")]
use geo::{polygon, point, Contains, LineString};

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
/// depending on given location (x,y) in ECEF [ddeg]
#[cfg(feature = "with-geo")]
pub fn sbas_selection_helper (point: geo::Point<f64>) -> Option<Augmentation> {
    let db = load_database();
    for (sbas, area) in db {
        if area.contains(&point) {
            return Some(sbas.clone())
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    #[cfg(feature = "with-geo")]
    fn test_sbas_selection() {
        // PARIS --> EGNOS
        let point :geo::Point<f64> = point!(
            x: 11.708425,
            y: 61.283695, 
        );
        assert_eq!(sbas_selection_helper(point).is_some(), true);
        let point :geo::Point<f64> = point!(
            x: 12.575282, 
            y: 60.330155,
        );
        assert_eq!(sbas_selection_helper(point).is_none(), true);
    }
}
