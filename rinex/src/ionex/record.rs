use crate::{merge, merge::Merge, prelude::*, split, split::Split};

use super::grid;

use crate::epoch;
use hifitime::Duration;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

pub(crate) fn is_new_tec_plane(line: &str) -> bool {
    line.contains("START OF TEC MAP")
}

pub(crate) fn is_new_rms_plane(line: &str) -> bool {
    line.contains("START OF RMS MAP")
}

/*
 * Don't know what Height maps are actually
 */
// pub(crate) fn is_new_height_map(line: &str) -> bool {
//     line.contains("START OF HEIGHT MAP")
// }

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TEC {
    /// TEC value
    pub tec: f64,
    /// RMS(tec)
    pub rms: Option<f64>,
}

pub type TECPlane = HashMap<(i32, i32), TEC>;

/// IONEX contains 2D (fixed altitude) or 3D Ionosphere Maps.
/// See [Rinex::ionex] and related feature for more information.
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
/// ```
pub type Record = BTreeMap<(Epoch, i32), TECPlane>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse map index from \"{0}\"")]
    MapIndexParsing(String),
    #[error("faulty epoch description")]
    EpochDescriptionError,
    #[error("bad grid definition")]
    BadGridDefinition(#[from] grid::Error),
    #[error("failed to parse {0} coordinates from \"{1}\"")]
    CoordinatesParsing(String, String),
    #[error("failed to parse epoch")]
    EpochParsing(#[from] epoch::ParsingError),
}

/*
 * Merges `rhs` into `lhs`
 */
fn merge_plane_mut(lhs: &mut TECPlane, rhs: &TECPlane) {
    for (coord, tec) in rhs {
        if lhs.get(coord).is_none() {
            lhs.insert(*coord, tec.clone());
        }
    }
}

/*
 * Parses following map, which can either be
 *  - a TEC map
 *  - an RMS tec map
 *  - an height map
 * Returns: Epoth(t), nth Map index, latitude, altitude and TEC plane accross longitudes
 */
pub(crate) fn parse_plane(
    content: &str,
    header: &mut Header,
    is_rms_plane: bool,
) -> Result<(usize, Epoch, i32, TECPlane), Error> {
    let lines = content.lines();
    let mut epoch = Epoch::default();
    let mut plane = TECPlane::with_capacity(128);

    // this can't fail at this point
    let ionex = header
        .ionex
        .as_mut()
        .expect("faulty ionex context: missing specific header definitions");

    // current {lat, lon} within current grid def.
    let mut latitude = 0_i32;
    let mut longitude = 0_i32;
    let mut altitude = 0_i32;
    let mut dlon = (ionex.grid.longitude.spacing * 1000.0) as i32;

    for line in lines {
        if line.len() > 60 {
            let (content, marker) = line.split_at(60);
            if marker.contains("START OF") {
                continue; // skip that one
            } else if marker.contains("END OF") && marker.contains("MAP") {
                let index = content.split_at(6).0;
                let index = index.trim();
                let index = u32::from_str_radix(index, 10)
                    .or(Err(Error::MapIndexParsing(index.to_string())))?;

                return Ok((index as usize, epoch, altitude, plane));
            } else if marker.contains("LAT/LON1/LON2/DLON/H") {
                // grid definition for next block
                let (_, rem) = content.split_at(2);

                let (lat, rem) = rem.split_at(6);
                let lat = lat.trim();
                let lat = f64::from_str(lat).or(Err(Error::CoordinatesParsing(
                    String::from("latitude"),
                    lat.to_string(),
                )))?;

                let (lon1, rem) = rem.split_at(6);
                let lon1 = lon1.trim();
                let lon1 = f64::from_str(lon1).or(Err(Error::CoordinatesParsing(
                    String::from("longitude"),
                    lon1.to_string(),
                )))?;

                let (_lon2, rem) = rem.split_at(6);
                //let lon2 = lon2.trim();
                //let lon2 = f64::from_str(lon2).or(Err(Error::CoordinatesParsing(
                //    String::from("longitude"),
                //    lon2.to_string(),
                //)))?;

                let (dlon_str, rem) = rem.split_at(6);
                let dlon_str = dlon_str.trim();
                let dlon_f64 = f64::from_str(dlon_str).or(Err(Error::CoordinatesParsing(
                    String::from("longitude"),
                    dlon_str.to_string(),
                )))?;

                let (h, _) = rem.split_at(6);
                let h = h.trim();
                let alt = f64::from_str(h).or(Err(Error::CoordinatesParsing(
                    String::from("altitude"),
                    h.to_string(),
                )))?;

                altitude = (alt.round() * 100.0_f64) as i32;
                latitude = (lat.round() * 1000.0_f64) as i32;
                longitude = (lon1.round() * 1000.0_f64) as i32;
                dlon = (dlon_f64.round() * 1000.0_f64) as i32;

                // debug
                // println!("NEW GRID : h: {} lat : {} lon : {}, dlon: {}", altitude, latitude, longitude, dlon);
            } else if marker.contains("EPOCH OF CURRENT MAP") {
                epoch = epoch::parse_utc(content)?.0;
            } else if marker.contains("EXPONENT") {
                // update current scaling
                if let Ok(e) = i8::from_str_radix(content.trim(), 10) {
                    ionex.exponent = e;
                }
            } else {
                // parsing TEC values
                for item in line.split_ascii_whitespace().into_iter() {
                    if let Ok(v) = i32::from_str_radix(item.trim(), 10) {
                        let mut value = v as f64;
                        // current scaling
                        value *= 10.0_f64.powf(ionex.exponent as f64);

                        let tec = match is_rms_plane {
                            true => {
                                TEC {
                                    tec: 0.0_f64, // DONT CARE
                                    rms: Some(value),
                                }
                            },
                            false => TEC {
                                tec: value,
                                rms: None,
                            },
                        };

                        plane.insert((latitude, longitude), tec);
                    }

                    longitude += dlon;
                    //debug
                    //println!("longitude: {}", longitude);
                }
            }
        } else {
            // less than 60 characters
            // parsing TEC values
            for item in line.split_ascii_whitespace().into_iter() {
                if let Ok(v) = i32::from_str_radix(item.trim(), 10) {
                    let mut value = v as f64;
                    // current scaling
                    value *= 10.0_f64.powf(ionex.exponent as f64);

                    let tec = match is_rms_plane {
                        true => {
                            TEC {
                                tec: 0.0_f64, // DONT CARE
                                rms: Some(value),
                            }
                        },
                        false => TEC {
                            tec: value,
                            rms: None,
                        },
                    };

                    plane.insert((latitude, longitude), tec);
                }

                longitude += dlon;
                //debug
                //println!("longitude: {}", longitude);
            }
        }
    }
    Ok((0, epoch, altitude, plane))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_tec_map() {
        assert_eq!(
            is_new_tec_plane(
                "1                                                      START OF TEC MAP"
            ),
            true
        );
        assert_eq!(
            is_new_tec_plane(
                "1                                                      START OF RMS MAP"
            ),
            false
        );
        assert_eq!(
            is_new_rms_plane(
                "1                                                      START OF RMS MAP"
            ),
            true
        );
        // assert_eq!(
        //     is_new_height_map(
        //         "1                                                      START OF HEIGHT MAP"
        //     ),
        //     true
        // );
    }
    //#[test]
    //fn test_merge_map2d() {
    //}
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
        for (eh, plane) in rhs {
            if let Some(lhs_plane) = self.get_mut(eh) {
                for (latlon, plane) in plane {
                    if let Some(tec) = lhs_plane.get_mut(latlon) {
                        if let Some(rms) = plane.rms {
                            if tec.rms.is_none() {
                                tec.rms = Some(rms);
                            }
                        }
                    } else {
                        lhs_plane.insert(*latlon, plane.clone());
                    }
                }
            } else {
                self.insert(*eh, plane.clone());
            }
        }
        Ok(())
    }
}

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let before = self
            .iter()
            .flat_map(|((e, h), plane)| {
                if *e < epoch {
                    Some(((*e, *h), plane.clone()))
                } else {
                    None
                }
            })
            .collect();
        let after = self
            .iter()
            .flat_map(|((e, h), plane)| {
                if *e >= epoch {
                    Some(((*e, *h), plane.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((before, after))
    }
    fn split_dt(&self, _duration: Duration) -> Result<Vec<Self>, split::Error> {
        Ok(Vec::new())
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
                TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e == epoch),
                _ => {}, // TargetItem:: does not apply
            },
            MaskOperand::NotEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e != epoch),
                _ => {}, // TargetItem:: does not apply
            },
            MaskOperand::GreaterEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e >= epoch),
                _ => {}, // TargetItem:: does not apply
            },
            MaskOperand::GreaterThan => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e > epoch),
                _ => {}, // TargetItem:: does not apply
            },
            MaskOperand::LowerEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e <= epoch),
                _ => {}, // TargetItem:: does not apply
            },
            MaskOperand::LowerThan => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e < epoch),
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
