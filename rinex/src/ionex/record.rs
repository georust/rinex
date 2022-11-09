use crate::{
    Header,
    Epoch, EpochFlag,
    merge, merge::Merge,
    split, split::Split,
    sampling::Decimation,
};

use super::{
    grid,
    GridLinspace,
};

use thiserror::Error;
use std::str::FromStr;
use std::collections::BTreeMap;

pub fn is_new_tec_map(line: &str) -> bool {
    line.contains("START OF TEC MAP") 
}

pub fn is_new_rms_map(line: &str) -> bool {
    line.contains("START OF RMS MAP") 
}

pub fn is_new_height_map (line: &str) -> bool {
    line.contains("START OF HEIGHT MAP") 
}

/// Returns true if given content describes the start of
/// a Ionosphere map.
pub fn is_new_map(line: &str) -> bool {
    is_new_tec_map(line) || is_new_rms_map(line) || is_new_height_map(line)
}

/// A Map is a list of estimates for
/// a given Latitude, Longitude, Altitude
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MapPoint {
    /// Latitude of this estimate
    latitude: f32,
    /// Longitude of this estimate
    longitude: f32,
    /// Altitude of this estimate
    altitude: f32,
    /// Actual estimate (scaling applied)
    value: f32,
}

pub type Map = Vec<MapPoint>;

/// `IONEX` record is sorted by epoch.
/// For each epoch, a TEC map is always given.
/// Possible RMS map and Height map may exist at a given epoch.
/// Ionosphere maps are always given in Earth fixed reference frames.
pub type Record = BTreeMap<Epoch, (Map, Option<Map>, Option<Map>)>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse map index")]
    ParseIndexError,
    #[error("faulty epoch description")]
    EpochDescriptionError,
    #[error("faulty longitude range definition")]
    LongitudeRangeError(#[from] grid::Error),  
}

/*
 * Parses following map, which can either be
 *  - a TEC map
 *  - an RMS tec map
 *  - an height map
 * defined for returned Epoch
 */
pub fn parse_map(header: &mut Header, content: &str) -> Result<(usize, Epoch, Map), Error> {
    let lines = content.lines();
    let mut epoch = Epoch::default();
    let mut map = Map::with_capacity(128); // result
    let mut latitude: f32 = 0.0; // current latitude
    let mut altitude: f32 = 0.0; // current altitude
    let mut ptr: usize = 0; // pointer in longitude space
    let mut linspace = GridLinspace::default(); // (longitude) linspace
    let mut ionex = header.ionex
        .as_mut()
        .expect("faulty ionex context: missing specific header definitions");
    for line in lines {
        if line.len() > 60 {
            let (content, marker) = line.split_at(60);
            if marker.contains("END OF") && marker.contains("MAP") {
                let index = content.split_at(6).0;
                if let Ok(u) = u32::from_str_radix(index.trim(), 10) {
                    return Ok((u as usize, epoch, map));
                } else {
                    return Err(Error::ParseIndexError);
                }

            } else if marker.contains("LAT/LON1/LON2/DLON/H") {
                // space coordinates definition for next block
                let (_, rem) = content.split_at(2);
                let (lat, rem) = rem.split_at(6);
                let (lon1, rem) = rem.split_at(6);
                let (lon2, rem) = rem.split_at(6);
                let (dlon, rem) = rem.split_at(6);
                let (h, rem) = rem.split_at(6);
                let lat = f32::from_str(lat.trim())
                    .expect("failed to parse grid latitude start point");
                let lon1 = f32::from_str(lon1.trim())
                    .expect("failed to parse longitude start point");
                let lon2 = f32::from_str(lon2.trim())
                    .expect("failed to parse longitude end point");
                let dlon = f32::from_str(dlon.trim())
                    .expect("failed to parse longitude grid spacing");
                let h = f32::from_str(h.trim())
                    .expect("failed to parse next grid altitude");
                linspace = GridLinspace::new(lon1, lon2, dlon)?;
                ptr = 0;

            } else if marker.contains("EPOCH OF CURRENT MAP") {
                // time definition
                let items: Vec<&str> = line.split_ascii_whitespace()
                    .collect();
                if items.len() != 6 {
                    return Err(Error::EpochDescriptionError);
                }
                if let Ok(y) = i32::from_str_radix(items[0].trim(), 10) {
                    if let Ok(m) = u32::from_str_radix(items[1].trim(), 10) {
                        if let Ok(d) = u32::from_str_radix(items[2].trim(), 10) {
                            if let Ok(hh) = u32::from_str_radix(items[3].trim(), 10) {
                                if let Ok(mm) = u32::from_str_radix(items[4].trim(), 10) {
                                    if let Ok(ss) = u32::from_str_radix(items[5].trim(), 10) {
                                        epoch = Epoch {
                                            date: chrono::NaiveDate::from_ymd(y,m,d)
                                                .and_hms(hh,mm,ss),
                                            flag: EpochFlag::default(),
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            } else if marker.contains("EXPONENT") {
                // scaling redefinition
                if let Ok(e) = i8::from_str_radix(content.trim(), 10) {
                    *ionex = ionex
                        .with_exponent(e); // scaling update
                }
            } else {
                // parsing TEC values 
                for item in line.split_ascii_whitespace().into_iter() {
                    if let Ok(v) = i32::from_str_radix(item.trim(), 10) {
                        // parse & apply correct scaling
                        let mut value = v as f32;
                        value *= 10.0_f32.powf(ionex.exponent as f32);
                        map.push(MapPoint {
                            latitude,
                            altitude,
                            longitude: linspace.start + linspace.spacing * ptr as f32,
                            value,
                        });
                        ptr += 1;
                    }
                }
            }
        } else {
            // parsing TEC values 
            for item in line.split_ascii_whitespace().into_iter() {
                if let Ok(v) = i32::from_str_radix(item.trim(), 10) {
                    // parse & apply correct scaling
                    let mut value = v as f32;
                    value *= 10.0_f32.powf(ionex.exponent as f32);
                    map.push(MapPoint {
                        latitude,
                        altitude,
                        longitude: linspace.start + linspace.spacing * ptr as f32,
                        value,
                    });
                    ptr += 1;
                }
            }
        }
    }
    Ok((0, epoch, map))
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_tec_map() {
        assert_eq!(is_new_tec_map("1                                                      START OF TEC MAP"), true); 
        assert_eq!(is_new_tec_map("1                                                      START OF RMS MAP"), false); 
        assert_eq!(is_new_rms_map("1                                                      START OF RMS MAP"), true); 
        assert_eq!(is_new_height_map("1                                                      START OF HEIGHT MAP"), true); 
    }
}

/*
impl Merge<Record> for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        for (epoch, (tec_map, rms_map, h_map)) in rhs.iter() {
            if let Some((ttec_map, rrms_map, hh_map)) = self.get_mut(epoch) {
                for coordinates in tec_map.iter() {
                    if ttec_map.contains(coordinates) {
                        let (coordinates, points) = coordinates;
                        for point in points {
                            for (ccoordinates, ppoints) in ttec_map.iter_mut() {
                                if coordinates == ccoordinates { // for this coordinate
                                    if !ppoints.contains(point) {
                                        // provide missing point
                                        ppoints.push(*point);
                                    }
                                }
                            }
                        }
                    } else { // provide previously missing coordinates
                        ttec_map.push(coordinates.clone());
                    }
                }
                if let Some(map) = rms_map {
                    if let Some(mmap) = rrms_map {
                        for coordinates in map.iter() {
                            if mmap.contains(coordinates) {
                                let (coordinates, points) = coordinates;
                                for point in points {
                                    for (ccoordinates, ppoints) in mmap.iter_mut() {
                                        if coordinates == ccoordinates { // for these coordinates
                                            if !ppoints.contains(point) {
                                                // provide missing point
                                                ppoints.push(*point);
                                            }
                                        }
                                    }
                                }
                            } else { // provide previous missing coordinates
                                mmap.push(coordinates.clone());
                            }
                        }
                    } else { // provide previously omitted RMS map
                        *rrms_map = Some(map.clone());
                    }
                }
                if let Some(map) = h_map {
                    if let Some(mmap) = hh_map {
                        for coordinates in map.iter() {
                            if mmap.contains(coordinates) {
                                let (coordinates, points) = coordinates;
                                for point in points {
                                    for (ccoordinates, ppoints) in mmap.iter_mut() {
                                        if coordinates == ccoordinates { // for these coordinates
                                            if !ppoints.contains(point) {
                                                // provide missing point
                                                ppoints.push(*point);
                                            }
                                        }
                                    }
                                }
                            } else { // provide previous missing coordinates
                                mmap.push(coordinates.clone());
                            }
                        }
                    } else { // provide previously omitted RMS map
                        *hh_map = Some(map.clone());
                    }
                }
            } else { // new epoch
                self.insert(*epoch, (tec_map.clone(), rms_map.clone(), h_map.clone()));
            }
        }
        Ok(())
    }
}

impl Split<Record> for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self.iter()
            .flat_map(|(k, v)| {
                if k.date < epoch.date {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self.iter()
            .flat_map(|(k, v)| {
                if k.date >= epoch.date {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((r0, r1))
    }
}

impl Decimation<Record> for Record {
    /// Decimates Self by desired factor
    fn decim_by_ratio_mut(&mut self, r: u32) {
        let mut i = 0;
        self.retain(|_, _| {
            let retained = (i % r) == 0;
            i += 1;
            retained
        });
    }
    /// Copies and Decimates Self by desired factor
    fn decim_by_ratio(&self, r: u32) -> Self {
        let mut s = self.clone();
        s.decim_by_ratio_mut(r);
        s
    }
    /// Decimates Self to fit minimum epoch interval
    fn decim_by_interval_mut(&mut self, interval: chrono::Duration) {
        let mut last_retained: Option<chrono::NaiveDateTime> = None;
        self.retain(|e, _| {
            if last_retained.is_some() {
                let dt = e.date - last_retained.unwrap();
                last_retained = Some(e.date);
                dt > interval
            } else {
                last_retained = Some(e.date);
                true // always retain 1st epoch
            }
        });
    }
    /// Copies and Decimates Self to fit minimum epoch interval
    fn decim_by_interval(&self, interval: chrono::Duration) -> Self {
        let mut s = self.clone();
        s.decim_by_interval_mut(interval);
        s
    }
}*/
