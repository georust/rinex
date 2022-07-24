use crate::epoch;
use std::collections::HashMap;

fn is_new_tec_map (line: &str) -> bool {
    line.contains("START OF TEC MAP") 
}
fn is_new_rms_map (line: &str) -> bool {
    line.contains("START OF RMS MAP") 
}
fn is_new_height_map (line: &str) -> bool {
    line.contains("START OF HEIGHT MAP") 
}
pub fn is_new_map (line: &str) -> bool {
       is_new_tec_map(line) 
    || is_new_rms_map(line)
    || is_new_height_map(line)
}

/// Map is one entry of the record item,
/// it is sorted by position: Lat/Long1/Long2/delta Longitude and altitude/height
pub type Map = Vec<(f32,f32,f32,f32,Vec<i32>)>;

pub struct RecordItem {
    /// Exponent / scaling attached to `tec` map
    exponent: Option<i8>,
    /// TEC map
    pub tec: Vec<Map>,
    /// Optionnal RMS map associated to `tec` map
    pub rms: Option<Vec<Map>>,
    /// Optionnal height map associated to `tec` map
    pub height: Option<Vec<Map>>,
}

/// `IONEX` record is a list of maps
pub type Record = HashMap<epoch::Epoch, RecordItem>;

impl RecordItem {
    fn get_exponent (&self) -> i8 {
        if let Some(e) = self.exponent {
            e
        } else {
            -1 // default value
        }
    }
    pub fn with_eponent (&mut self, e: i8) {
        self.exponent = Some(e)
    }
    /*pub fn add_rms_map (&mut self, map: Map) {
        if let Some(mut rms) = self.rms {
            rms.push(map)
        } else {
            self.rms = Some(vec![map])
        }
    }
    pub fn add_height_map (&mut self, map: Map) {
        if let Some(mut m) = self.height {
            m.push(map)
        } else {
            self.height = Some(vec![map])
        }
    }*/
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_map() {
        assert_eq!(is_new_map("1                                                      START OF TEC MAP   "), true); 
        assert_eq!(is_new_map("1                                                      START OF TEC MAP   "), true); 
    }
}
