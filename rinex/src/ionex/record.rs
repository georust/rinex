use crate::{
    gnss_time::TimeScaling, merge, merge::Merge, prelude::*, sampling::Decimation, split,
    split::Split,
};

use super::{grid, GridLinspace};

use hifitime::Duration;
use std::collections::BTreeMap;
use std::str::FromStr;
use thiserror::Error;

pub(crate) fn is_new_tec_map(line: &str) -> bool {
    line.contains("START OF TEC MAP")
}

pub(crate) fn is_new_rms_map(line: &str) -> bool {
    line.contains("START OF RMS MAP")
}

pub(crate) fn is_new_height_map(line: &str) -> bool {
    line.contains("START OF HEIGHT MAP")
}

/// Returns true if given content describes the start of
/// a Ionosphere map.
pub(crate) fn is_new_map(line: &str) -> bool {
    is_new_tec_map(line) || is_new_rms_map(line) || is_new_height_map(line)
}

/// A Map is a list of estimates for
/// a given Latitude, Longitude, Altitude
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MapPoint {
    /// Latitude of this estimate
    pub latitude: f32,
    /// Longitude of this estimate
    pub longitude: f32,
    /// Altitude of this estimate
    pub altitude: f32,
    /// Actual estimate (scaling applied)
    pub value: f32,
}

pub type Map = Vec<MapPoint>;

/// `IONEX` record is sorted by epoch.
/// For each epoch, a TEC map is always given.
/// Possible RMS map and Height map may exist at a given epoch.
/// Ionosphere maps are always given in Earth fixed reference frames.
/// ```
/// use rinex::prelude::*;
/// use rinex::ionex::*;
/// let rinex = Rinex::from_file("../test_resources/IONEX/V1/CKMG0020.22I.gz")
///     .unwrap();
/// assert_eq!(rinex.is_ionex(), true);
/// assert_eq!(rinex.is_ionex_2d(), true);
/// if let Some(params) = rinex.header.ionex {
///     assert_eq!(params.grid.height.start, 350.0); // 2D: record uses
///     assert_eq!(params.grid.height.end, 350.0); // fixed altitude
///     assert_eq!(params.grid.latitude.start, 87.5);
///     assert_eq!(params.grid.latitude.end, -87.5);
///     assert_eq!(params.grid.latitude.spacing, -2.5); // latitude granularity (degrees)
///     assert_eq!(params.grid.longitude.start, -180.0);
///     assert_eq!(params.grid.longitude.end, 180.0);
///     assert_eq!(params.grid.longitude.spacing, 5.0); // longitude granularity (degrees)
///     assert_eq!(params.exponent, -1); // data scaling. May vary accross epochs.
///                             // so this is only the last value encountered
///     assert_eq!(params.elevation_cutoff, 0.0);
///     assert_eq!(params.mapping, None); // no mapping function
/// }
/// let record = rinex.record.as_ionex()
///     .unwrap();
/// for (epoch, (tec, rms, height)) in record {
///     // RMS map never provided in this file
///     assert_eq!(rms.is_none(), true);
///     // 2D IONEX: height maps never provided
///     assert_eq!(height.is_none(), true);
///     // We only get TEC maps
///     // when using TEC values, we previously applied all required scalings
///     for point in tec {
///         let lat = point.latitude; // in ddeg
///         let lon = point.longitude; // in ddeg
///         let alt = point.altitude; // in km
///         let value = point.value; // correctly scaled ("exponent")
///     }
/// }
/// ```
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
pub(crate) fn parse_map(header: &mut Header, content: &str) -> Result<(usize, Epoch, Map), Error> {
    let lines = content.lines();
    let mut epoch = Epoch::default();
    let mut map = Map::with_capacity(128); // result
    let mut latitude: f32 = 0.0; // current latitude
    let mut altitude: f32 = 0.0; // current altitude
    let mut ptr: usize = 0; // pointer in longitude space
    let mut linspace = GridLinspace::default(); // (longitude) linspace
    let ionex = header
        .ionex
        .as_mut()
        .expect("faulty ionex context: missing specific header definitions");
    for line in lines {
        if line.len() > 60 {
            let (content, marker) = line.split_at(60);
            if marker.contains("START OF") {
                continue; // skip that one
            } else if marker.contains("END OF") && marker.contains("MAP") {
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
                let (h, _) = rem.split_at(6);
                latitude =
                    f32::from_str(lat.trim()).expect("failed to parse grid latitude start point");
                let lon1 =
                    f32::from_str(lon1.trim()).expect("failed to parse longitude start point");
                let lon2 = f32::from_str(lon2.trim()).expect("failed to parse longitude end point");
                let dlon =
                    f32::from_str(dlon.trim()).expect("failed to parse longitude grid spacing");
                altitude = f32::from_str(h.trim()).expect("failed to parse next grid altitude");
                linspace = GridLinspace::new(lon1, lon2, dlon)?;
                ptr = 0;
            } else if marker.contains("EPOCH OF CURRENT MAP") {
                // time definition
                let items: Vec<&str> = content.split_ascii_whitespace().collect();
                if items.len() != 6 {
                    return Err(Error::EpochDescriptionError);
                }
                if let Ok(y) = i32::from_str_radix(items[0].trim(), 10) {
                    if let Ok(m) = u8::from_str_radix(items[1].trim(), 10) {
                        if let Ok(d) = u8::from_str_radix(items[2].trim(), 10) {
                            if let Ok(hh) = u8::from_str_radix(items[3].trim(), 10) {
                                if let Ok(mm) = u8::from_str_radix(items[4].trim(), 10) {
                                    if let Ok(ss) = u8::from_str_radix(items[5].trim(), 10) {
                                        epoch = Epoch::from_gregorian_utc(y, m, d, hh, mm, ss, 0);
                                    }
                                }
                            }
                        }
                    }
                }
            } else if marker.contains("EXPONENT") {
                // scaling redefinition
                if let Ok(e) = i8::from_str_radix(content.trim(), 10) {
                    *ionex = ionex.with_exponent(e); // scaling update
                }
            } else {
                // parsing TEC values
                for item in line.split_ascii_whitespace().into_iter() {
                    if let Ok(v) = i32::from_str_radix(item.trim(), 10) {
                        // parse & apply correct scaling
                        let mut value = v as f32;
                        value *= 10.0_f32.powf(ionex.exponent as f32);
                        map.push(MapPoint {
                            latitude: latitude,
                            longitude: linspace.start + linspace.spacing * ptr as f32,
                            altitude: altitude,
                            value: value,
                        });
                        ptr += 1;
                    }
                }
            }
        } else {
            // less than 60 characters
            // parsing TEC values
            for item in line.split_ascii_whitespace().into_iter() {
                if let Ok(v) = i32::from_str_radix(item.trim(), 10) {
                    // parse & apply correct scaling
                    let mut value = v as f32;
                    value *= 10.0_f32.powf(ionex.exponent as f32);
                    map.push(MapPoint {
                        latitude: latitude,
                        longitude: linspace.start + linspace.spacing * ptr as f32,
                        altitude: altitude,
                        value: value,
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
        assert_eq!(
            is_new_tec_map(
                "1                                                      START OF TEC MAP"
            ),
            true
        );
        assert_eq!(
            is_new_tec_map(
                "1                                                      START OF RMS MAP"
            ),
            false
        );
        assert_eq!(
            is_new_rms_map(
                "1                                                      START OF RMS MAP"
            ),
            true
        );
        assert_eq!(
            is_new_height_map(
                "1                                                      START OF HEIGHT MAP"
            ),
            true
        );
    }
}

impl Merge<Record> for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, _rhs: &Self) -> Result<(), merge::Error> {
        /*
        for (epoch, maps) in rhs.iter() {
            if let (tec, Some(rms), Some(h)) = maps {
                if let Some(maps) = self.get_mut(epoch) {
                    let ((ttec, rrms, hh)) = maps;
                    if rrms.is_none() {
                        // RMS map now provided for this epoch
                        rrms = Some(map);
                    }
                    if hh.is_none() {
                        // Height map now provided for this epoch
                        hh = Some(map);
                    }
                } else { // new epoch
                    self.insert(*epoch, (tec, rms, h));
                }
            }
        }*/
        Ok(())
    }
}

impl Split<Record> for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if *k < epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if *k >= epoch {
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
    fn decim_by_interval_mut(&mut self, interval: Duration) {
        let mut last_retained: Option<Epoch> = None;
        self.retain(|e, _| {
            if last_retained.is_some() {
                let dt = *e - last_retained.unwrap();
                last_retained = Some(*e);
                dt > interval
            } else {
                last_retained = Some(*e);
                true // always retain 1st epoch
            }
        });
    }
    /// Copies and Decimates Self to fit minimum epoch interval
    fn decim_by_interval(&self, interval: Duration) -> Self {
        let mut s = self.clone();
        s.decim_by_interval_mut(interval);
        s
    }
    fn decim_match_mut(&mut self, rhs: &Self) {
        self.retain(|e, _| rhs.get(e).is_some());
    }
    fn decim_match(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.decim_match_mut(&rhs);
        s
    }
}

impl TimeScaling<Record> for Record {
    fn convert_timescale(&mut self, ts: TimeScale) {
        self.iter_mut()
            .map(|(k, v)| (k.in_time_scale(ts), v))
            .count();
    }
    fn with_timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.convert_timescale(ts);
        s
    }
}
