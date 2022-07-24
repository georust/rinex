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

/// `IONEX` record is a list of maps indexed by `epoch`
pub type Record = HashMap<epoch::Epoch, Maps>;

/// A Map represents either a TEC, RMS or H map,
/// with associated lat/lon1/lon2/d/h values
pub type Map = (f32,f32,f32,f32,f32,Vec<i32>);

#[derive(Clone, Debug, Default)]
pub struct Maps {
    /// TEC maps
    pub tec: Vec<Map>,
    /// Optionnal RMS maps associated to `tec` map,
    /// most of the time not provided
    pub rms: Vec<Map>,
    /// Optionnal height map associated to `tec` map
    /// most of the time not provided
    pub height: Vec<Map>,
}

impl Maps {
    /// Returns (properly scaled) TEC maps
    pub fn tec_maps (&self) -> Vec<(f32, f32, f32, f32, f32, Vec<f32>)> {
        self
            .tec
            .iter()
            .map(|(lat, lon1, lon2, dlon, h, data)| {
                (
                *lat, 
                *lon1, 
                *lon2, 
                *dlon, 
                *h, 
                data.iter()
                    .map(|value| {
                        *value as f32
                    })
                    .collect()
                )
            })
            .collect()
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
