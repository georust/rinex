use hifitime::Epoch;
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

use crate::{
    doris::Station,
    epoch::{parse_in_timescale, EpochFlag, ParsingError as EpochParsingError},
    header::Header,
    observable::Observable,
    prelude::{Duration, TimeScale},
};

/// DORIS RINEX Record content.
/// Measurements are stored by Kind, by Station and by TAI sampling instant.
pub type Record = BTreeMap<(Epoch, EpochFlag), BTreeMap<Station, HashMap<Observable, f64>>>;

/// Returns true if following line matches a new DORIS measurement
pub(crate) fn is_new_epoch(line: &str) -> bool {
    line.starts_with('>')
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse epoch")]
    EpochError(#[from] EpochParsingError),
    #[error("failed to parse data")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

#[cfg(feature = "serde")]
use serde::Serialize;

/// DORIS measurement parsing process
pub(crate) fn parse_epoch(
    header: &Header,
    content: &str,
) -> Result<
    (
        (Epoch, EpochFlag),
        BTreeMap<Station, HashMap<Observable, f64>>,
    ),
    Error,
> {
    let mut obs_idx = 0usize;
    let mut epoch = Epoch::default();
    let mut flag = EpochFlag::default();
    let mut station = Option::<Station>::None;
    let mut buffer = BTreeMap::<Station, HashMap<Observable, f64>>::new();

    let doris = header
        .doris
        .as_ref()
        .expect("missing header field(s): badly formed DORIS RINEX");

    let observables = &doris.observables;
    let stations = &doris.stations;

    assert!(
        stations.len() > 0,
        "badly formed DORIS RINEX: no stations defined"
    );
    assert!(
        observables.len() > 0,
        "badly formed DORIS RINEX: on observables defined"
    );

    for (lindex, line) in content.lines().enumerate() {
        match lindex {
            0 => {
                /* 1st line gives TAI timestamp, flag, clock offset */
                let line = line.split_at(2).1; // "> "
                let offset = "YYYY MM DD HH MM SS.NNNNNNNNN  0".len();
                let (date, rem) = line.split_at(offset);
                (epoch, flag) = parse_in_timescale(date, TimeScale::TAI)?;
            },
            _ => {
                let (id, remainder) = line.split_at(5);

                if obs_idx == 0 {
                    // parse station identifier
                    assert!(id.len() > 1, "badly formed DORIS station identifier");
                    let key = id[1..]
                        .trim()
                        .parse::<u16>()
                        .unwrap_or_else(|e| panic!("failed to identify DORIS station: {:?}", e));

                    station = stations
                        .iter()
                        .filter(|station| station.key == key)
                        .reduce(|k, _| k)
                        .cloned();
                }

                let identified_station =
                    station.as_ref().expect("failed to identify DORIS station");

                // consume this line
                let mut offset = 5;
                let mut max_offset = line.len();
                while offset < line.len() {
                    let content = &line[offset..std::cmp::min(max_offset, offset + 16)];
                    println!("OBS \"{}\"", content); //DEBUG

                    let value = content
                        .trim()
                        .parse::<f64>()
                        .unwrap_or_else(|e| panic!("failed to parse float value: {:?}", e));

                    let observable = observables.get(obs_idx).unwrap_or_else(|| {
                        panic!(
                            "failed to determine observable for {:?}({:?}) @ {}",
                            identified_station, epoch, obs_idx
                        )
                    });

                    if let Some(station) = buffer.get_mut(identified_station) {
                        station.insert(observable.clone(), value);
                    } else {
                        let mut inner =
                            HashMap::from_iter([(Observable::default(), value)].into_iter());
                        buffer.insert(identified_station.clone(), inner);
                    }

                    offset += 16;
                    obs_idx += 1;
                }
                if obs_idx == observables.len() {
                    obs_idx = 0;
                    station = None;
                }
            },
        }
    }
    Ok(((epoch, flag), buffer))
}

#[cfg(feature = "processing")]
use crate::preprocessing::*;

#[cfg(feature = "processing")]
impl Preprocessing for Record {
    fn filter(&self, filter: Filter) -> Self {
        let mut s = self.clone();
        s.filter_mut(filter);
        s
    }
    fn filter_mut(&mut self, filter: Filter) {
        match filter {
            Filter::Mask(mask) => self.mask_mut(mask),
            Filter::Decimation(filter) => match filter.dtype {
                DecimationType::DecimByRatio(r) => {
                    if filter.target.is_none() {
                        self.decimate_by_ratio_mut(r);
                        return; // no need to proceed further
                    }

                    let item = filter.target.unwrap();

                    // apply mask to retain desired subset
                    let mask = MaskFilter {
                        item: item.clone(),
                        operand: MaskOperand::Equals,
                    };

                    // and decimate
                    let subset = self.mask(mask).decimate_by_ratio(r);

                    // adapt self's subset to new data rates
                    decimate_data_subset(self, &subset, &item);
                },
                DecimationType::DecimByInterval(dt) => {
                    if filter.target.is_none() {
                        self.decimate_by_interval_mut(dt);
                        return; // no need to proceed further
                    }

                    let item = filter.target.unwrap();

                    // apply mask to retain desired subset
                    let mask = MaskFilter {
                        item: item.clone(),
                        operand: MaskOperand::Equals,
                    };

                    // and decimate
                    let subset = self.mask(mask).decimate_by_interval(dt);

                    // adapt self's subset to new data rates
                    decimate_data_subset(self, &subset, &item);
                },
            },
            _ => {},
        }
    }
}

/*
 * Decimates only a given record subset
 */
#[cfg(feature = "processing")]
fn decimate_data_subset(record: &mut Record, subset: &Record, target: &TargetItem) {
    match target {
        TargetItem::ClockItem => {
            /*
             * Remove clock fields from self
             * where it should now be missing
             */
            for (epoch, _) in record.iter_mut() {
                //if subset.get(epoch).is_none() {
                //    // should be missing
                //    // *clk = None; // now missing
                //}
            }
        },
        TargetItem::SvItem(svs) => {
            /*
             * Remove SV observations where it should now be missing
             */
            for (epoch, _) in record.iter_mut() {
                //if subset.get(epoch).is_none() {
                //    // should be missing
                //    for sv in svs.iter() {
                //        vehicles.remove(sv); // now missing
                //    }
                //}
            }
        },
        TargetItem::ObservableItem(obs_list) => {
            /*
             * Remove given observations where it should now be missing
             */
            for (epoch, _) in record.iter_mut() {
                //if subset.get(epoch).is_none() {
                //    // should be missing
                //    for (_sv, observables) in vehicles.iter_mut() {
                //        observables.retain(|observable, _| !obs_list.contains(observable));
                //    }
                //}
            }
        },
        TargetItem::ConstellationItem(constells_list) => {
            /*
             * Remove observations for given constellation(s) where it should now be missing
             */
            for (epoch, _) in record.iter_mut() {
                //if subset.get(epoch).is_none() {
                //    // should be missing
                //    vehicles.retain(|sv, _| {
                //        let mut contained = false;
                //        for constell in constells_list.iter() {
                //            if sv.constellation == *constell {
                //                contained = true;
                //                break;
                //            }
                //        }
                //        !contained
                //    });
                //}
            }
        },
        TargetItem::SNRItem(_) => unimplemented!("decimate_data_subset::snr"),
        _ => {},
    }
}

#[cfg(feature = "processing")]
impl Decimate for Record {
    fn decimate_by_ratio_mut(&mut self, r: u32) {
        let mut i = 0;
        self.retain(|_, _| {
            let retained = (i % r) == 0;
            i += 1;
            retained
        });
    }
    fn decimate_by_ratio(&self, r: u32) -> Self {
        let mut s = self.clone();
        s.decimate_by_ratio_mut(r);
        s
    }
    fn decimate_by_interval_mut(&mut self, interval: Duration) {
        let mut last_retained = Option::<Epoch>::None;
        self.retain(|(e, _), _| {
            if let Some(last) = last_retained {
                let dt = *e - last;
                if dt >= interval {
                    last_retained = Some(*e);
                    true
                } else {
                    false
                }
            } else {
                last_retained = Some(*e);
                true // always retain 1st epoch
            }
        });
    }
    fn decimate_by_interval(&self, interval: Duration) -> Self {
        let mut s = self.clone();
        s.decimate_by_interval_mut(interval);
        s
    }
    fn decimate_match_mut(&mut self, rhs: &Self) {
        self.retain(|e, _| rhs.get(e).is_some());
    }
    fn decimate_match(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.decimate_match_mut(rhs);
        s
    }
}

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
                TargetItem::EpochFlagItem(flag) => self.retain(|(_, f), _| *f == flag),
                TargetItem::ObservableItem(filter) => {
                    self.retain(|_, stations| {
                        stations.retain(|_, obs| {
                            obs.retain(|code, _| filter.contains(code));
                            !obs.is_empty()
                        });
                        !stations.is_empty()
                    });
                },
                _ => {}, //TODO: some other types could apply, like SNR..
            },
            MaskOperand::NotEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|(e, _), _| *e != epoch),
                TargetItem::EpochFlagItem(flag) => self.retain(|(_, f), _| *f != flag),
                TargetItem::ObservableItem(filter) => {
                    self.retain(|_, stations| {
                        stations.retain(|_, obs| {
                            obs.retain(|code, _| !filter.contains(code));
                            !obs.is_empty()
                        });
                        !stations.is_empty()
                    });
                },
                _ => {}, //TODO: some other types could apply, like SNR..
            },
            _ => {},
        }
    }
}

#[cfg(test)]
mod test {
    use super::{is_new_epoch, parse_epoch};
    use crate::{doris::HeaderFields as DorisHeader, doris::Station, Header, Observable};
    use std::str::FromStr;
    #[test]
    fn new_epoch() {
        for (desc, expected) in [
            (
                "> 2024 01 01 00 00 28.999947700  0  2       -0.151364695 0 ",
                true,
            ),
            (
                "> 2023 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                true,
            ),
            (
                "  2023 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                false,
            ),
            (
                "  2022 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                false,
            ),
            ("test", false),
        ] {
            assert_eq!(is_new_epoch(desc), expected);
        }
    }
    #[test]
    fn valid_epoch() {
        let mut header = Header::default();
        let mut doris = DorisHeader::default();
        for obs in ["L1", "L2", "C1", "C2", "W1", "W2", "F", "P", "T", "H"] {
            let obs = Observable::from_str(obs).unwrap();
            doris.observables.push(obs);
        }
        for station in [
            "D01  THUB THULE                         43001S005  3   0",
            "D02  SVBC NY-ALESUND II                 10338S004  4   0",
        ] {
            let station = Station::from_str(station).unwrap();
            doris.stations.push(station);
        }
        header.doris = Some(doris);

        for desc in ["> 2024 01 01 00 00 28.999947700  0  2       -0.151364695 0 
D01  -3237877.052    -2291024.044    21903595.62311  21903633.08011      -113.100 7
          -98.400 7       437.801        1002.000 1       -20.000 1        82.000 1
D02  -2069899.788     -407871.014     4677242.25714   4677392.20614      -119.050 7
         -111.000 7       437.801        1007.000 0        -2.000 0        74.000 0"]
        {
            let epoch = parse_epoch(&header, desc);
            assert!(epoch.is_ok(), "failed to parse DORIS epoch");
        }
    }
}