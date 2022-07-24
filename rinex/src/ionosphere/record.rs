use crate::epoch;
use thiserror::Error;
//use std::str::FromStr;
use std::collections::HashMap;

fn is_new_tec_map (line: &str) -> bool {
    line.contains("START OF TEC MAP") 
}

/*fn is_new_rms_map (line: &str) -> bool {
    line.contains("START OF RMS MAP") 
}

fn is_new_height_map (line: &str) -> bool {
    line.contains("START OF HEIGHT MAP") 
}*/

pub fn is_new_map (line: &str) -> bool {
       is_new_tec_map(line) 
//    || is_new_rms_map(line)
//    || is_new_height_map(line)
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

#[derive(Debug, Error)]
pub enum Error {
    #[error("nothing wrong")]
    NoError,
}

/// Builds list of identified maps and associated epoch 
pub fn build_record_entry (_content: &str, _exponent: i8) -> Result<(epoch::Epoch, Maps), Error> {
/*
    let lines = content.lines();
    let mut exp = exponent.clone();
    let mut epoch = epoch::Epoch::default();
    let (mut lat, mut lon1, mut lon2, mut dlon, mut h) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    for line in lines {
        let (content, marker) = line.split_at(60);
        println!("line: {}\n", marker);
        if marker.contains("LAT/LON1/LON2/DLON/H") {
            let items : Vec<&str> = content
                .split_ascii_whitespace()
                .collect();
            if let Ok(f) = f32::from_str(items[0].trim()) {
                lat = f
            }
            if let Ok(f) = f32::from_str(items[1].trim()) {
                lon1 = f
            }
            if let Ok(f) = f32::from_str(items[2].trim()) {
                lon2 = f
            }
            if let Ok(f) = f32::from_str(items[3].trim()) {
                dlon = f
            }
            if let Ok(f) = f32::from_str(items[4].trim()) {
                h = f
            }

        } else if marker.contains("EPOCH OF CURRENT MAP") {
            let items : Vec<&str> = content
                .trim()
                .split_ascii_whitespace()
                .collect();
            let mut datestr = items[0].to_owned(); // Y
            datestr.push_str(" ");
            datestr.push_str(items[1]); // m
            datestr.push_str(" ");
            datestr.push_str(items[2]); // d
            datestr.push_str(" ");
            datestr.push_str(items[3]); // h
            datestr.push_str(" ");
            datestr.push_str(items[4]); // m
            datestr.push_str(" ");
            datestr.push_str(items[5]); // s
            println!("DATESTR \"{}\"", datestr);
            if let Ok(e) = epoch::str2date(&datestr) {
                epoch.date = e
            }
        } else if marker.contains("EXPONENT") {
            if let Ok(e) = i8::from_str_radix(content.trim(), 10) {
                exp = e
            }

        } else {
            // --> inside map
        }
    }
 */
    Ok((epoch::Epoch::default(), Maps {
        tec: Vec::new(),
        rms: Vec::new(),
        height: Vec::new(),
    }))
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_map() {
        assert_eq!(is_new_map("1                                                      START OF TEC MAP   "), true); 
        assert_eq!(is_new_map("1                                                      START OF RMS MAP   "), false); 
    }

    #[test]
    fn test_ionex_v1_example1() {
        let content =
"     1                                                      START OF TEC MAP    
  1995    10    15     0     0     0                        EPOCH OF CURRENT MAP
    -3                                                      EXPONENT            
    85.0   0.0 355.0   5.0 200.0                            LAT/LON1/LON2/DLON/H
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000                                        
    80.0   0.0 355.0   5.0 200.0                            LAT/LON1/LON2/DLON/H
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000                                        
...                                                                             
   -85.0   0.0 355.0   5.0 200.0                            LAT/LON1/LON2/DLON/H
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000                                        
     5                                                      END OF TEC MAP     "; 
        let entry = build_record_entry(content, -1);
        println!("{:#?}", entry);
    }
}
