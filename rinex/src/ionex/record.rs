use crate::{
    algorithm::{
        Filter, Interpolate, Mask, MaskFilter, MaskOperand, Preprocessing, Scale, ScalingFilter,
        ScalingType, TargetItem,
    },
    gnss_time::GnssTime,
    merge,
    merge::Merge,
    prelude::*,
    split,
    split::Split,
};

use super::GridLinspace;

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
    is_new_tec_map(line)
    // || is_new_rms_map(line)
    // || is_new_height_map(line)
}

/// A map of TEC values
pub type Map1d = Vec<(f64, f64)>;
/// A 2D map of TEC values, and possible RMS error
pub type Map2d = Vec<(f64, Map1d)>;
/// A 3D map of TEC values, and possible RMS error
pub type Map3d = Vec<(f64, Map2d)>;

/// `IONEX` record is sorted by epoch.
/// For each epoch, a TEC map is always given.
/// We currently do not parse RMS errors nor height maps.
/// Each TEC value is stored per (latitude (ddeg°), longitude(ddeg°) and altitude (h))
/// therefore a 3D representation is supported.
/// Ionosphere maps are always given in Earth fixed reference frames.
/// ```
/// use rinex::prelude::*;
/// use rinex::ionex::*;
/// let rinex = Rinex::from_file("../test_resources/IONEX/V1/CKMG0020.22I.gz")
///     .unwrap();
/// assert_eq!(rinex.is_ionex(), true);
/// assert_eq!(rinex.is_2d_ionex(), true);
/// if let Some(ionex) = rinex.header.ionex {
///     // map grid characteristics
///     assert_eq!(ionex.map_grid.h_grid.start, 350.0); // 2D IONEX: fixed altitude
///     assert_eq!(ionex.map_grid.h_grid.start, ionex.map_grid.h_grid.end); // 2D IONEX
///     assert_eq!(ionex.map_grid.lat_grid.start, 87.5);
///     assert_eq!(ionex.map_grid.lat_grid.end, -87.5);
///     assert_eq!(ionex.map_grid.lat_grid.spacing, -2.5); // latitude granularity (ddeg)
///     assert_eq!(ionex.map_grid.lon_grid.start, -180.0);
///     assert_eq!(ionex.map_grid.lon_grid.end, 180.0);
///     assert_eq!(ionex.map_grid.lon_grid.spacing, 5.0); // longitude granularity (ddeg)
/// }
/// let record = rinex.record.as_ionex()
///     .unwrap();
/// // Browse TEC values per altitude, latitude and longitude
/// for (epoch, altitudes) in record {
///     for (z, latitudes) in altitudes {
///         for (lat, longitudes) in latitudes {
///             for (lon, tec) in longitudes {
///                 // tec: is the TEC estimate,
///                 // rms error is not available yet
///             }
///         }
///     }
/// }
/// ```
pub type Record = BTreeMap<Epoch, Map3d>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse map index")]
    ParseIndexError,
    #[error("faulty epoch description")]
    EpochDescriptionError,
}
/*
 * Parse data points per longitude coordinates.
 * Returns (entry_index, Epoch, latitude, altitude and Vec<(longitude, datapoints)>.
 * "entry_index" is present in the file, and is helpful since this Record definition
 * is Vector<> based, we can only index with an integer value (not complex data).
 * Data points can either be TEC or RMS map: this is determined at a higher level.
 * Mut header access is required to support real time scaling adjustment.
 */
pub(crate) fn parse_map_entry(
    header: &mut Header,
    content: &str,
) -> Result<(usize, Epoch, Map3d), Error> {
    let lines = content.lines();
    let mut epoch = Epoch::default(); // to be parsed in this paragraph
    let mut latitude: f64 = 0.0; // latitude, to be returned
    let mut altitude: f64 = 0.0; // h, to be returned
    let mut longitude: f64 = 0.0; // current longitude: to be updated
    let mut d_lon: f64 = 0.0; // d_lon: difference in longitude ddeg° between two data points
    let mut map1d = Map1d::new(); //map1d: points per longitude
    let mut map2d = Map2d::new(); //map2d: longitudes points per latitude
    let ionex = header
        .ionex
        .as_mut()
        .expect("faulty ionex context: missing specific header definitions");
    for line in lines {
        if line.len() > 60 {
            let (content, marker) = line.split_at(60);
            if marker.contains("START OF") {
                continue; // header: skip it
            } else if marker.contains("END OF") && marker.contains("MAP") {
                // conclude this entry
                let index = content.split_at(6).0;
                if let Ok(u) = u32::from_str_radix(index.trim(), 10) {
                    if map1d.len() > 0 {
                        map2d.push((latitude, map1d.clone()));
                    }
                    let map3d: Map3d = vec![(altitude, map2d.clone())];
                    return Ok((u as usize, epoch, map3d));
                } else {
                    return Err(Error::ParseIndexError);
                }
            } else if marker.contains("LAT/LON1/LON2/DLON/H") {
                // append previous content, if it exists
                if map1d.len() > 0 {
                    // avoids pushing empty content on first entry
                    map2d.push((latitude, map1d.clone()));
                    map1d.clear();
                }
                // grid definition for next block
                let (_, rem) = content.split_at(2);
                let (lat, rem) = rem.split_at(6);
                let (lon1, rem) = rem.split_at(6);
                let (_lon2, rem) = rem.split_at(6);
                let (dlon, rem) = rem.split_at(6);
                let (h, _) = rem.split_at(6);
                latitude =
                    f64::from_str(lat.trim()).expect("failed to parse grid latitude start point");
                longitude =
                    f64::from_str(lon1.trim()).expect("failed to parse longitude start point");
                d_lon = f64::from_str(dlon.trim()).expect("failed to parse longitude grid spacing");
                altitude = f64::from_str(h.trim()).expect("failed to parse next grid altitude");
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
                    *ionex = ionex.with_exponent(e); // update scaling
                }
            } else {
                // parsing TEC values
                for item in line.split_ascii_whitespace().into_iter() {
                    if let Ok(v) = i32::from_str_radix(item.trim(), 10) {
                        // parse & apply correct scaling
                        let mut value = v as f64;
                        value *= 10.0_f64.powf(ionex.exponent as f64);
                        map1d.push((longitude, value));
                        longitude += d_lon;
                    }
                }
            }
        } else {
            // less than 60 characters: we're inside the paragraph
            for item in line.split_ascii_whitespace().into_iter() {
                if let Ok(v) = i32::from_str_radix(item.trim(), 10) {
                    // parse & apply correct scaling
                    let mut value = v as f64;
                    value *= 10.0_f64.powf(ionex.exponent as f64);
                    map1d.push((longitude, value));
                    longitude += d_lon;
                }
            }
        }
    }
    unreachable!("missing \"END OF MAP\" marker");
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
    //#[test]
    //fn test_merge_map2d() {
    //    let mut lhs = vec![
    //        MapPoint {
    //            latitude: 0.0,
    //            longitude: 0.0,
    //            altitude: 0.0,
    //            value: 1.0,
    //        },
    //        MapPoint {
    //            latitude: 0.0,
    //            longitude: 10.0,
    //            altitude: 0.0,
    //            value: 2.0,
    //        },
    //        MapPoint {
    //            latitude: 0.0,
    //            longitude: 20.0,
    //            altitude: 0.0,
    //            value: 3.0,
    //        },
    //        MapPoint {
    //            latitude: 10.0,
    //            longitude: 0.0,
    //            altitude: 0.0,
    //            value: 4.0,
    //        },
    //        MapPoint {
    //            latitude: 10.0,
    //            longitude: 10.0,
    //            altitude: 0.0,
    //            value: 5.0,
    //        },
    //        MapPoint {
    //            latitude: 10.0,
    //            longitude: 20.0,
    //            altitude: 0.0,
    //            value: 6.0,
    //        },
    //    ];
    //    let rhs = vec![
    //        MapPoint {
    //            latitude: 0.0,
    //            longitude: 0.0,
    //            altitude: 0.0,
    //            value: 0.0,
    //        },
    //        MapPoint {
    //            latitude: 5.0,
    //            longitude: 0.0,
    //            altitude: 0.0,
    //            value: 1.0,
    //        },
    //        MapPoint {
    //            latitude: 10.0,
    //            longitude: 0.0,
    //            altitude: 0.0,
    //            value: 0.0,
    //        },
    //        MapPoint {
    //            latitude: 10.0,
    //            longitude: 25.0,
    //            altitude: 0.0,
    //            value: 6.0,
    //        },
    //    ];
    //    let expected = vec![
    //        MapPoint {
    //            latitude: 0.0,
    //            longitude: 0.0,
    //            altitude: 0.0,
    //            value: 1.0,
    //        },
    //        MapPoint {
    //            latitude: 0.0,
    //            longitude: 10.0,
    //            altitude: 0.0,
    //            value: 2.0,
    //        },
    //        MapPoint {
    //            latitude: 0.0,
    //            longitude: 20.0,
    //            altitude: 0.0,
    //            value: 3.0,
    //        },
    //        MapPoint {
    //            latitude: 10.0,
    //            longitude: 0.0,
    //            altitude: 0.0,
    //            value: 4.0,
    //        },
    //        MapPoint {
    //            latitude: 10.0,
    //            longitude: 10.0,
    //            altitude: 0.0,
    //            value: 5.0,
    //        },
    //        MapPoint {
    //            latitude: 10.0,
    //            longitude: 20.0,
    //            altitude: 0.0,
    //            value: 6.0,
    //        },
    //        MapPoint {
    //            latitude: 5.0,
    //            longitude: 0.0,
    //            altitude: 0.0,
    //            value: 1.0,
    //        },
    //        MapPoint {
    //            latitude: 10.0,
    //            longitude: 25.0,
    //            altitude: 0.0,
    //            value: 6.0,
    //        },
    //    ];
    //    map_merge3d_mut(&mut lhs, &rhs);
    //    assert_eq!(&lhs, &expected);
    //}
}

use super::Ionex;

impl Ionex for Record {
    fn latitudes(&self) -> Vec<f64> {
        if let Some((_e0, z_maps)) = self.first_key_value() {
            if let Some((_z0, z0_map)) = z_maps.get(0) {
                return z0_map.iter().map(|x| x.0).collect::<Vec<f64>>();
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
    fn longitudes(&self) -> Vec<f64> {
        if let Some((_, z_maps)) = self.first_key_value() {
            if let Some((_z0, lat_map)) = z_maps.get(0) {
                if let Some((lat0, lon0_map)) = lat_map.get(0) {
                    return lon0_map.iter().map(|x| x.0).collect::<Vec<f64>>();
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
    fn max(&self) -> (Epoch, f64, f64, f64, f64) {
        let mut epoch_max = Epoch::default();
        let mut z_max = -f64::INFINITY;
        let mut lat_max = -f64::INFINITY;
        let mut lon_max = -f64::INFINITY;
        let mut tec_max = -f64::INFINITY;
        for (e, z_maps) in self {
            for (z, lat_maps) in z_maps {
                for (lat, lon_maps) in lat_maps {
                    for (lon, tec) in lon_maps {
                        if *tec > tec_max {
                            tec_max = *tec;
                            z_max = *z;
                            lat_max = *lat;
                            lon_max = *lon;
                            epoch_max = *e;
                        }
                    }
                }
            }
        }
        (epoch_max, lat_max, lon_max, z_max, tec_max)
    }
    fn min(&self) -> (Epoch, f64, f64, f64, f64) {
        let mut epoch_min = Epoch::default();
        let mut z_min = f64::INFINITY;
        let mut lat_min = f64::INFINITY;
        let mut lon_min = f64::INFINITY;
        let mut tec_min = f64::INFINITY;
        for (e, z_maps) in self {
            for (z, lat_maps) in z_maps {
                for (lat, lon_maps) in lat_maps {
                    for (lon, tec) in lon_maps {
                        if *tec < tec_min {
                            tec_min = *tec;
                            z_min = *z;
                            lat_min = *lat;
                            lon_min = *lon;
                            epoch_min = *e;
                        }
                    }
                }
            }
        }
        (epoch_min, lat_min, lon_min, z_min, tec_min)
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
        for (epoch, rhs_map3d) in rhs {
            if let Some(lhs_map3d) = self.get_mut(epoch) {
                //TODO
            } else {
                // new epoch: insert as is
                self.insert(*epoch, rhs_map3d.clone());
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

impl Preprocessing for Record {
    fn filter(&self, f: Filter) -> Self {
        let mut s = self.clone();
        s.filter_mut(f);
        s
    }
    fn filter_mut(&mut self, f: Filter) {
        match f {
            Filter::Mask(mask) => self.mask_mut(mask),
            Filter::Scaling(f) => self.scale_mut(f),
            Filter::Smoothing(_) => unimplemented!("filter:smoothing on ionex"),
            Filter::Decimation(_) => unimplemented!("filter:decimation on ionex"),
            Filter::Interp(filter) => self.interpolate_mut(filter.series),
        }
    }
}

impl Interpolate for Record {
    fn interpolate(&self, series: TimeSeries) -> Self {
        let mut s = self.clone();
        s.interpolate_mut(series);
        s
    }
    fn interpolate_mut(&mut self, _series: TimeSeries) {
        unimplemented!("ionex:record:interpolate_mut()")
    }
}

impl Scale for Record {
    fn scale(&self, scaling: ScalingFilter) -> Self {
        let mut s = self.clone();
        s.scale_mut(scaling);
        s
    }
    fn scale_mut(&mut self, scaling: ScalingFilter) {
        match scaling.stype {
            ScalingType::Offset(b) => self.offset_mut(b),
            ScalingType::Rescale(bins) => self.rescale_mut(bins),
        }
    }
    fn offset(&self, b: f64) -> Self {
        let mut s = self.clone();
        s.offset_mut(b);
        s
    }
    fn offset_mut(&mut self, b: f64) {
        for (_e, z_maps) in self.iter_mut() {
            for (_z, lat_maps) in z_maps.iter_mut() {
                for (_lat, lon_maps) in lat_maps.iter_mut() {
                    for (_lon, tec) in lon_maps.iter_mut() {
                        *tec += b;
                    }
                }
            }
        }
    }
    fn rescale(&self, bins: usize) -> Self {
        let mut s = self.clone();
        s.rescale_mut(bins);
        s
    }
    fn rescale_mut(&mut self, bins: usize) {
        // 1. determine max|TEC|
        let (_, _, _, _, min) = self.min();
        let (_, _, _, _, max) = self.max();
        let dtot = max - min;
        let dtec = dtot / bins as f64;
        // min   <dtec>    max
        // |----|-----|-----|
        for (_, altitudes) in self.iter_mut() {
            for (_, latitudes) in altitudes.iter_mut() {
                for (_, longitudes) in latitudes.iter_mut() {
                    for (_, tec) in longitudes.iter_mut() {
                        for i in 0..bins {
                            let d_i = dtec * ((i as f64) + 1.0);
                            let d_j = dtec * ((i as f64) + 2.0);
                            if *tec >= d_i && *tec <= d_j {
                                // this value is contained in that space
                                // -> map to closest limit
                                let dest_i = (*tec - d_i).abs();
                                let dest_j = (*tec - d_j).abs();
                                if dest_i < dest_j {
                                    *tec = d_i;
                                } else {
                                    *tec = d_j;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
