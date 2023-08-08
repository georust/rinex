use crate::{gnss_time::GnssTime, merge, merge::Merge, prelude::*, split, split::Split};

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
    pub latitude: f64,
    /// Longitude of this estimate
    pub longitude: f64,
    /// Altitude of this estimate
    pub altitude: f64,
    /// Actual estimate (scaling applied)
    pub value: f64,
}

pub type Map = Vec<MapPoint>;

/*
 * Merges `rhs` into `lhs` in up to 3 dimensions
 */
fn map_merge3d_mut(lhs: &mut Map, rhs: &Map) {
    for rhs_p in rhs {
        let mut found = false;
        for lhs_p in lhs.into_iter() {
            found |= (lhs_p.latitude == rhs_p.latitude)
                && (lhs_p.longitude == rhs_p.longitude)
                && (lhs_p.altitude == rhs_p.altitude);
            if found {
                break;
            }
        }
        if !found {
            lhs.push(rhs_p.clone());
        }
    }
}

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
    let mut latitude: f64 = 0.0; // current latitude
    let mut altitude: f64 = 0.0; // current altitude
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
                    f64::from_str(lat.trim()).expect("failed to parse grid latitude start point");
                let lon1 =
                    f64::from_str(lon1.trim()).expect("failed to parse longitude start point");
                let lon2 = f64::from_str(lon2.trim()).expect("failed to parse longitude end point");
                let dlon =
                    f64::from_str(dlon.trim()).expect("failed to parse longitude grid spacing");
                altitude = f64::from_str(h.trim()).expect("failed to parse next grid altitude");
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
                        let mut value = v as f64;
                        value *= 10.0_f64.powf(ionex.exponent as f64);
                        map.push(MapPoint {
                            latitude: latitude,
                            longitude: linspace.start + linspace.spacing * ptr as f64,
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
                    let mut value = v as f64;
                    value *= 10.0_f64.powf(ionex.exponent as f64);
                    map.push(MapPoint {
                        latitude: latitude,
                        longitude: linspace.start + linspace.spacing * ptr as f64,
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
    #[test]
    fn test_merge_map2d() {
        let mut lhs = vec![
            MapPoint {
                latitude: 0.0,
                longitude: 0.0,
                altitude: 0.0,
                value: 1.0,
            },
            MapPoint {
                latitude: 0.0,
                longitude: 10.0,
                altitude: 0.0,
                value: 2.0,
            },
            MapPoint {
                latitude: 0.0,
                longitude: 20.0,
                altitude: 0.0,
                value: 3.0,
            },
            MapPoint {
                latitude: 10.0,
                longitude: 0.0,
                altitude: 0.0,
                value: 4.0,
            },
            MapPoint {
                latitude: 10.0,
                longitude: 10.0,
                altitude: 0.0,
                value: 5.0,
            },
            MapPoint {
                latitude: 10.0,
                longitude: 20.0,
                altitude: 0.0,
                value: 6.0,
            },
        ];
        let rhs = vec![
            MapPoint {
                latitude: 0.0,
                longitude: 0.0,
                altitude: 0.0,
                value: 0.0,
            },
            MapPoint {
                latitude: 5.0,
                longitude: 0.0,
                altitude: 0.0,
                value: 1.0,
            },
            MapPoint {
                latitude: 10.0,
                longitude: 0.0,
                altitude: 0.0,
                value: 0.0,
            },
            MapPoint {
                latitude: 10.0,
                longitude: 25.0,
                altitude: 0.0,
                value: 6.0,
            },
        ];
        let expected = vec![
            MapPoint {
                latitude: 0.0,
                longitude: 0.0,
                altitude: 0.0,
                value: 1.0,
            },
            MapPoint {
                latitude: 0.0,
                longitude: 10.0,
                altitude: 0.0,
                value: 2.0,
            },
            MapPoint {
                latitude: 0.0,
                longitude: 20.0,
                altitude: 0.0,
                value: 3.0,
            },
            MapPoint {
                latitude: 10.0,
                longitude: 0.0,
                altitude: 0.0,
                value: 4.0,
            },
            MapPoint {
                latitude: 10.0,
                longitude: 10.0,
                altitude: 0.0,
                value: 5.0,
            },
            MapPoint {
                latitude: 10.0,
                longitude: 20.0,
                altitude: 0.0,
                value: 6.0,
            },
            MapPoint {
                latitude: 5.0,
                longitude: 0.0,
                altitude: 0.0,
                value: 1.0,
            },
            MapPoint {
                latitude: 10.0,
                longitude: 25.0,
                altitude: 0.0,
                value: 6.0,
            },
        ];
        map_merge3d_mut(&mut lhs, &rhs);
        assert_eq!(&lhs, &expected);
    }
}

impl Merge for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        for (epoch, maps) in rhs {
            let (tec, rms, h) = maps;
            if let Some(lhs_maps) = self.get_mut(epoch) {
                let (lhs_tec, lhs_rms, lhs_h) = lhs_maps;

                map_merge3d_mut(&mut lhs_tec.to_vec(), tec);

                if let Some(map) = rms {
                    if let Some(lhs_map) = lhs_rms {
                        map_merge3d_mut(&mut lhs_map.to_vec(), map);
                    } else {
                        *lhs_rms = Some(map.to_vec()); // RMS map now provided
                    }
                }

                if let Some(map) = h {
                    if let Some(lhs_map) = lhs_h {
                        map_merge3d_mut(&mut lhs_map.to_vec(), map);
                    } else {
                        *lhs_h = Some(map.to_vec()); // H map now provided
                    }
                }
            } else {
                // new epoch
                self.insert(*epoch, (tec.to_vec(), rms.clone(), h.clone()));
            }
        }
        Ok(())
    }
}

impl Split for Record {
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
    fn split_dt(&self, _duration: Duration) -> Result<Vec<Self>, split::Error> {
        Ok(Vec::new())
    }
}

impl GnssTime for Record {
    fn timeseries(&self, dt: Duration) -> TimeSeries {
        let epochs: Vec<_> = self.keys().collect();
        TimeSeries::inclusive(
            **epochs.get(0).expect("failed to determine first epoch"),
            **epochs
                .get(epochs.len() - 1)
                .expect("failed to determine last epoch"),
            dt,
        )
    }
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

#[cfg(feature = "processing")]
use crate::preprocessing::*;

#[cfg(feature = "processing")]
impl Mask for Record {
    fn mask(&self, mask: MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(mask);
        s
    }
    fn mask_mut(&mut self, mask: MaskFilter) {
        match mask.operand {
            MaskOperand::Equals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e == epoch),
                _ => {}, // TargetItem:: does not apply
            },
            MaskOperand::NotEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e != epoch),
                _ => {}, // TargetItem:: does not apply
            },
            MaskOperand::GreaterEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e >= epoch),
                _ => {}, // TargetItem:: does not apply
            },
            MaskOperand::GreaterThan => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e > epoch),
                _ => {}, // TargetItem:: does not apply
            },
            MaskOperand::LowerEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e <= epoch),
                _ => {}, // TargetItem:: does not apply
            },
            MaskOperand::LowerThan => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e < epoch),
                _ => {}, // TargetItem:: does not apply
            },
        }
    }
}

#[cfg(feature = "processing")]
impl Preprocessing for Record {
    fn filter(&self, f: Filter) -> Self {
        let mut s = self.clone();
        s.filter_mut(f);
        s
    }
    fn filter_mut(&mut self, f: Filter) {
        match f {
            Filter::Mask(mask) => self.mask_mut(mask),
            Filter::Smoothing(_) => todo!(),
            Filter::Decimation(_) => todo!(),
            Filter::Interp(filter) => self.interpolate_mut(filter.series),
        }
    }
}

#[cfg(feature = "processing")]
impl Interpolate for Record {
    fn interpolate(&self, series: TimeSeries) -> Self {
        let mut s = self.clone();
        s.interpolate_mut(series);
        s
    }
    fn interpolate_mut(&mut self, _series: TimeSeries) {
        unimplemented!("ionex:record:interpolate()")
    }
}
