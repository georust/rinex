use std::str::FromStr;
use strum_macros::EnumString;
pub mod record;

pub use record::{
    Record,
    is_new_map,
    parse_epoch,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[derive(EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Mapping function used in when determining this IONEX
pub enum MappingFunction {
    /// 1/cos(z)
    #[strum(serialize = "COSZ")]
    CosZ,
    /// Q-factor
    #[strum(serialize = "QFAC")]
    QFac,
}

/// Satellite system or theoretical model used
/// in this determination
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[derive(EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum System {
    /// BENt
    BEN,
    /// ENVisat
    ENV,
    /// ERS
    ERS,
    /// Geostationnary satellites
    GEO,
    /// GNSS or Glonass
    GNS,
    /// GPS
    GPS,
    /// IRI
    IRI,
    /// Mixed / combined GNSS
    MIX,
    /// NNNS transit
    NNS,
    /// TOPex / poseidon
    TOP,
}

impl Default for System {
    fn default() -> Self {
        Self::GPS
    }
}

/// Grid definition element,
/// start - end values with increment
#[derive(Debug, Clone, Default)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Grid3dElement {
    /// Grid start value
    pub start: f32,
    /// Grid end value
    pub end: f32,
    /// Grid increment value
    pub increment: f32,
}

impl From<(f32,f32,f32)> for Grid3dElement {
    fn from (tuple:(f32,f32,f32)) -> Self {
        Self {
            start: tuple.0,
            end: tuple.1,
            increment: tuple.2,
        }
    }
}

/// Grid definition in terms of
/// latitude, longitude and altitude
#[derive(Debug, Clone, Default)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Grid3d {
    /// Latitude
    pub latitude: Grid3dElement,
    /// Longitude
    pub longitude: Grid3dElement,
    /// Height / altitude
    pub height: Grid3dElement,
}

/// `IONEX` specific header fields
#[derive(Debug, Clone, Default)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// System used or theoretical model used
    pub system: System,
    /// Known Differential PR Code Biases
    pub dcb: HashMap<Sv, (f64,f64)>, 
    /// It is highly recommended to give a brief description
    /// of the technique, model.. description is not a 
    /// general purpose comment
    pub description: Option<String>,
    /// Mapping function adopted for TEC determination,
    /// if None: No mapping function, e.g altimetry
    pub mapping: Option<MappingFunction>,
    /// TEC/RMS maps dimension
    pub map_dimension: u8,
    /// Mean earth radis or bottom of height grid, in km
    pub base_radius: f32,
    /// Equidistant grid definitions,
    /// in terms of Latitude, Longitude and Height/Altitude
    pub grid: Grid3d,
    /// Minimum elevation angle, in degrees,
    /// 0.0 if not known, 90.0 for altimetry
    pub elevation_cutoff: f32,
    /// Verbose description of observables used in determination,
    /// if None: this is a theoretical model
    pub observables: Option<String>,
    /// Number of stations that contributed to this model/these measurements
    pub nb_stations: Option<u32>,
    /// Number of satellites that contributed to this model/these measurements
    pub nb_satellites: Option<u32>,
}

impl HeaderFields {
    pub fn with_system (&self, system: &str) -> Self {
        let mut s = self.clone();
        if let Ok(system) = System::from_str(system) {
            s.system = system
        }
        s
    }
    pub fn with_description(&self, desc: &str) -> Self {
        let mut s = self.clone();
        if let Some(ref mut d) = s.description {
            d.push_str(desc)
        } else {
            s.description = Some(desc.to_string())
        }
        s
    }
    pub fn with_mapping_function(&self, func: &str) -> Self {
        let mut s = self.clone();
        if let Ok(func) = MappingFunction::from_str(func) {
            s.mapping = Some(func);
        } else {
            s.mapping = None
        }
        s
    }
    pub fn with_elevation (&self, e: f32) -> Self {
        let mut s = self.clone();
        s.elevation_cutoff = e;
        s
    }
    pub fn with_observables (&self, o: &str) -> Self {
        let mut s = self.clone();
        if o.len() > 0 {
            s.observables = Some(o.to_string())
        }
        s
    }
    /// Returns true if this Ionosphere Maps describes
    /// a theoretical model, not measured data
    pub fn is_theoretical_model (&self) -> bool {
        self.observables.is_some()
    }
    pub fn with_stations (&self, n: u32) -> Self {
        let mut s = self.clone();
        s.nb_stations = Some(n);
        s
    }
    pub fn with_satellites (&self, n: u32) -> Self {
        let mut s = self.clone();
        s.nb_satellites = Some(n);
        s
    }
    pub fn with_base_radius (&self, b: f32) -> Self {
        let mut s = self.clone();
        s.base_radius = b;
        s
    }
    pub fn with_map_dimension (&self, d: u8) -> Self {
        let mut s = self.clone();
        s.map_dimension = d;
        s
    }
    /// Define grid in terms of latitude
    pub fn with_grid_latitude (&self, l: (f32,f32,f32)) -> Self {
        let mut s = self.clone();
        s.grid.latitude = l.into();
        s
    }
    /// Define grid in terms of longitude
    pub fn with_grid_longitude (&self, l: (f32,f32,f32)) -> Self {
        let mut s = self.clone();
        s.grid.longitude = l.into();
        s
    }
    /// Define grid in terms of altitude 
    pub fn with_grid_height (&self, h: (f32,f32,f32)) -> Self {
        let mut s = self.clone();
        s.grid.height = h.into();
        s
    }
    /// Adds given diffrential code biases, to the list of known DCBs
    pub fn with_dcb(&self, sv: Sv, value: f64, rms: f64) -> Self {
        let mut s = self.clone();
        s.dcb.insert(*sv, (value, rms));
        s
    }
}

/*
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct MapCoordinates {
    pub lat: f64,
    pub lon1: f64,
    pub lon2: f64,
    pub dlon: f64,
    pub h: f64,
}

impl Default for MapCoordinates {
    fn default() -> MapCoordinates {
        MapCoordinates {
            lat: 0.0_f64,
            lon1: 0.0_f64,
            lon2: 0.0_f64,
            dlon: 0.0_f64,
            h: 0.0_f64,
        }
    }
}

/// A map is a vector of data for a given position
pub type Map = BTreeMap<Position, Vec<i32>>;

/// IONEX Record Payload,
/// is at least a Ionospheric Tec Map,
/// and possibly an Tec RMS map and a Height map
pub struct Data {
    pub tec_map: Map,
    pub rms_map: Option<Map>,
    pub height_map: Option<Map>,
}

/// IONEX Record is a list of `Data` indexed by `epoch`
pub type Record = BTreeMap<(epoch::Epoch, Data)>;

/// Builds a new ionex map
/// from at least a group of TEC MAP,
/// it may comprise an RMS map and a height map
pub fn build_record_entry (content: &str) -> Result<(epoch::Epoch, Data), RecordError> {
    let mut epoch = epoch::default();
    let mut tec_map = Map::new();
    let mut rms_map = Map::new();
    let mut height_map = Map::new();
    let mut is_rms = false;
    let mut is_height = false;
    let lines = content.lines();
    let mut pos = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    for line in lines {
        let (content, marker) = line.split_at(60);
        if marker.contains("EPOCH OF CURRENT MAP") {
            epoch = str2date(content.trim())?; 
            is_rms = false;
            is_height = false;
        
        } else if marker.contains("LAT/LON1/LON2/DLON/H") {
            let (lat, rem) = content.split_at(5);//TODO
            let (lon1, rem) = rem.split_at(5);//TODO
            let (lon2, rem) = rem.split_at(5);//TODO
            let (dlon, rem) = rem.split_at(5);//TODO
            let (h, rem) = rem.split_at(5);//TODO
            pos = (
                f64::from_str(lat.trim())?,
                f64::from_str(lon1.trim())?,
                f64::from_str(lon2.trim())?,
                f64::from_str(dlon.trim())?,
                f64::from_str(h.trim())?,
            );

        } else if marker.contains("START OF RMS MAP") {
            is_rms = true;
            is_height = false;
        
        } else if marker.contains("START OF HEIGHT MAP") {
            is_rms = false;
            is_height = true;

        } else { // --> inside map
            let items : Vec<&str> = content
                .split_ascii_whitespace()
                .collect();
            for item in items.iter() {
                let value = i32::from_str_radix(item.trim(), 10)?; 
                if is_rms {
                    if let Some(mut m) = rms_map.get(epoch) {
                        m.insert(value);
                    } else {
                        rms_map.insert(epoch, vec![value]);
                    }
                } else if is_height {
                    if let Some(mut m) = height_map.get(epoch) {
                        m.insert(value);
                    } else {
                        height_map.insert(epoch, vec![value]);
                    }
                } else {
                    if let Some(mut m) = tec_map.get(epoch) {
                        m.insert(value);
                    } else {
                        tec_map.insert(epoch, vec![value]);
                    }
                }
            }
        }
    }
    let data = Data {
        tec_map: map,
        rms_map: {
            if rms_map.len() > 0 {
                Some(rms_map)
            } else {
                None
            }
        },
        height_map: {
            if height_map.len() > 0 {
                Some(height_map)
            } else {
                None
            }
        },
    };
    Ok((epoch, data))
}
*/

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_mapping_func() {
        let content = "COSZ";
        let func = MappingFunction::from_str(content);
        assert_eq!(func.is_ok(), true);
        assert_eq!(func.unwrap(), MappingFunction::CosZ);
        let content = "QFAC";
        let func = MappingFunction::from_str(content);
        assert_eq!(func.is_ok(), true);
        assert_eq!(func.unwrap(), MappingFunction::QFac);
        let content = "DONT";
        let func = MappingFunction::from_str(content);
        assert_eq!(func.is_err(), true);
    }
}
