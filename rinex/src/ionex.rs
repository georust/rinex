pub fn is_new_tec_map (line: &str) -> bool {
    line.contains("START OF TEC MAP")
}

#[derive(Debug, Clone)]
#[derive(PartialEq, Eq, Hash)]
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
