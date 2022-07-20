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
use std::iter::FromIterator;
use wkt::{Geometry, Wkt, WktFloat};
use geo::{polygon, point, Contains, LineString};

/// SBAS augmentation system selection helper,
/// returns most approriate Augmentation system
/// depending on given location (x,y) in ECEF [ddeg]
#[cfg(feature = "with-geo")]
pub fn sbas_selection_helper (pos: (f64,f64)) -> Option<Augmentation> {
    let point : geo::Point = pos.into();
    let boundaries :Vec<(Augmentation, geo::Polygon)> = vec![
        (Augmentation::WAAS, polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ]),
    ];
    for (sbas, area) in boundaries.iter() {
        if area.exterior().contains(&point) {
            return Some(sbas.clone())
        }
    }
    None
}

fn wkt_line_string_to_geo<T> (line_string: &wkt::types::LineString<T>) -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    LineString::from_iter(line_string.0
        .iter()
        .map(|coord| (coord.x, coord.y)))
}

fn line_string<T>(name: &str) -> LineString<T>
where 
    T: WktFloat + Default + FromStr,
{
    let mut res = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    res.push("fixtures");
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    #[cfg(feature = "with-geo")]
    fn test_sbas_selection() {
        // PARIS --> EGNOS
        //let pos = (0.5, 0.5);
        //assert_eq!(sbas_selection_helper(pos).is_some(), true);
        let norway = geo::Polygon::<f64>::new(
            line_string("norway_main.wkt"), // exterior
            vec![]); // interior

        let point :geo::Point<f64> = point!(
            x: 9.789122, 
            y: 62.418818,
        );
        assert_eq!(norway.contains(&point), true);
        
        let point :geo::Point<f64> = point!(
            x: 12.542482, 
            y: 60.382449,
        );
        assert_eq!(norway.contains(&point), true);
        
        let point :geo::Point<f64> = point!(
            x: 12.575282, 
            y: 60.330155,
        );
        assert_eq!(norway.contains(&point), false);
    }
}
