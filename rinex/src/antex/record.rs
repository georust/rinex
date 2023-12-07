use gnss::prelude::SV;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

use super::{
    antenna::SvAntennaParsingError, Antenna, AntennaSpecific, Calibration, CalibrationMethod,
    Cospar, Frequency, Pattern, RxAntenna, SvAntenna,
};
use crate::{carrier, merge, merge::Merge, Carrier, Epoch};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Returns true if this line matches
/// the beginning of a `epoch` for ATX file (special files),
/// this is not really an epoch but rather a group of dataset
/// for this given antenna, there is no sampling data attached to it.
pub(crate) fn is_new_epoch(content: &str) -> bool {
    content.contains("START OF ANTENNA")
}

/// We currently do not support azimuth dependent phase patterns.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum AntennaPhasePattern {
    AzimuthIndependentPattern(Vec<f64>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct FrequencyDependentData {
    /// Eccentricities of the mean APC as NEU coordinates in millimeters.
    /// The offset position is either relative to
    /// Antenna Reference point (ARP), if this is an [`RxAntenna`],
    /// or the Spacecraft Mass Center, if this is an [`SvAntenna`].
    pub apc_eccentricity: (f64, f64, f64),
    /// Antenna Phase Pattern.
    /// We currently do not support Azimuth Dependent phase patterns.
    pub phase_pattern: AntennaPhasePattern,
}

/// ANTEX RINEX record content.
/// Data is a list of Antenna containing several [Frequency] items.
/// We do not parse RMS frequencies at the moment, but it will
/// easily be unlocked in near future.
/*TODO
/// Record browsing example:
/// ```
/// // grab ATX RINEX
///
/// // grab record
/// let record = record.as_antex()
///    .unwrap();
/// // browse antennas
/// for (antenna, frequencies) in record.iter() {
///   let calibration = antenna.calibration;
///   // several calibration methods exist
///   if calibration.method == CalibrationMethod::Chamber {
///     // calibration is certified
///     // from `calibration.valid_from` (chrono::NaiveDateTime)
///     // until `calibration.valid_until` (chrono::NaiveDateTime)
///   }
///   // calibration process informations
///   assert_eq!(calibration.agency, "Some agency");
///   assert_eq!(calibration.date, "DateTime description");
///   // antenna information
///   assert_eq!(antenna.sn, "Serial Number");
///   assert_eq!(antenna.dazi, 1.0);
///   for frequency in frequencies.iter() {
///     for pattern in frequency.patterns {
///         assert_eq!(pattern.is_azimuth_dependent(), true);
///         let Some((azimuth, phase_pattern)) = pattern.azimuth_dependent() {
///             for raw_phase in phase_patter.iter() {
///                 // raw phase pattern data
///             }
///         }
///     }
///   }
/// }
/// ```
*/
pub type Record = BTreeMap<Antenna, HashMap<Carrier, FrequencyDependentData>>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown PCV \"{0}\"")]
    UnknownPcv(String),
    #[error("Failed to parse carrier frequency")]
    ParseCarrierError(#[from] carrier::Error),
    #[error("sv parsing error")]
    SvParsing(#[from] gnss::sv::ParsingError),
    #[error("sv antenna parsing error")]
    SvAntennaParsing(#[from] SvAntennaParsingError),
}

/// Parses entire Antenna block
/// and all inner frequency entries
pub(crate) fn parse_antenna(content: &str) -> Result<Antenna, Error> {
    let lines = content.lines();
    let mut antenna = Antenna::default();

    //let mut frequency = Frequency::default();
    //let mut frequencies: Vec<Frequency> = Vec::new();
    for line in lines {
        let (content, marker) = line.split_at(60);
        if marker.contains("TYPE / SERIAL NO") {
            let (ant_igs, rem) = content.split_at(20);
            let (block1, rem) = rem.split_at(20);
            let (block2, rem) = rem.split_at(10);
            let (block3, rem) = rem.split_at(10);

            let (block1, block2, block3) = (block1.trim(), block2.trim(), block3.trim());
            /*
             * SV/RX antenna determination
             */
            let specificities = match block2.is_empty() && block3.is_empty() {
                false => AntennaSpecific::SvAntenna(SvAntenna {
                    igs_type: ant_igs.trim().to_string(),
                    sv: SV::from_str(block1)?,
                    cospar: Cospar::from_str(block3)?,
                }),
                true => AntennaSpecific::RxAntenna(RxAntenna {
                    igs_type: ant_igs.trim().to_string(),
                    serial_number: {
                        if !block1.is_empty() && !block1.eq("NONE") {
                            Some(block1.to_string())
                        } else {
                            None
                        }
                    },
                }),
            };
            antenna = antenna.with_specificities(specificities);
        } else if marker.contains("METH / BY / # / DATE") {
            let (method, rem) = content.split_at(20);
            let (agency, rem) = rem.split_at(20);
            let (_, rem) = rem.split_at(10); // N#
            let (date, _) = rem.split_at(10);
            let cal = Calibration {
                method: CalibrationMethod::from_str(method.trim()).unwrap(),
                agency: agency.trim().to_string(),
                date: Epoch::default(), // TODO
            };
            antenna = antenna.with_calibration(cal)
        } else if marker.contains("DAZI") {
            let dazi = content.split_at(20).0.trim();
            if let Ok(dazi) = f64::from_str(dazi) {
                antenna = antenna.with_dazi(dazi)
            }
        } else if marker.contains("# OF FREQUENCIES") {
            if let Ok(nb) = marker.parse::<u32>() {
                antenna.nb_frequencies = nb as usize;
            }
        } else if marker.contains("END OF ANTENNA") {
            break; // end of this block, considered as an `epoch`
                   // if we make a parallel with other types of RINEX
        }
        //    } else if marker.contains("ZEN1 / ZEN2 / DZEN") {
        //        let (zen1, rem) = content.split_at(8);
        //        let (zen2, rem) = rem.split_at(6);
        //        let (dzen, _) = rem.split_at(6);
        //        if let Ok(zen1) = f64::from_str(zen1.trim()) {
        //            if let Ok(zen2) = f64::from_str(zen2.trim()) {
        //                if let Ok(dzen) = f64::from_str(dzen.trim()) {
        //                    antenna = antenna.with_zenith(zen1, zen2, dzen)
        //                }
        //            }
        //        }
        //    } else if marker.contains("VALID FROM") {
        //        if let Ok(epoch) = Epoch::from_str(content.trim()) {
        //            antenna = antenna.with_valid_from(epoch)
        //        }
        //    } else if marker.contains("VALID UNTIL") {
        //        if let Ok(epoch) = Epoch::from_str(content.trim()) {
        //            antenna = antenna.with_valid_until(epoch)
        //        }
        //    } else if marker.contains("SINEX CODE") {
        //        let sinex = content.split_at(10).0;
        //        antenna = antenna.with_sinex_code(sinex.trim())
        //    } else if marker.contains("START OF FREQUENCY") {
        //        let svnn = content.split_at(10).0;
        //        let carrier = carrier::Carrier::from_sv(SV::from_str(svnn.trim())?)?;
        //        frequency = Frequency::default().with_carrier(carrier);
        //    } else if marker.contains("NORTH / EAST / UP") {
        //        let (north, rem) = content.split_at(10);
        //        let (east, rem) = rem.split_at(10);
        //        let (up, _) = rem.split_at(10);
        //        if let Ok(north) = f64::from_str(north.trim()) {
        //            if let Ok(east) = f64::from_str(east.trim()) {
        //                if let Ok(up) = f64::from_str(up.trim()) {
        //                    frequency = frequency
        //                        .with_northern_eccentricity(north)
        //                        .with_eastern_eccentricity(east)
        //                        .with_upper_eccentricity(up)
        //                }
        //            }
        //        }
        //    } else if marker.contains("END OF FREQUENCY") {
        //        frequencies.push(frequency.clone())
        //    } else {
        //        // Inside frequency
        //        // Determine type of pattern
        //        let (content, rem) = line.split_at(8);
        //        let values: Vec<f64> = rem
        //            .split_ascii_whitespace()
        //            .map(|item| {
        //                if let Ok(f) = f64::from_str(item.trim()) {
        //                    f
        //                } else {
        //                    panic!("failed to \"{}\" \"{}\"", content, marker);
        //                }
        //            })
        //            .collect();
        //        if line.contains("NOAZI") {
        //            frequency = frequency.add_pattern(Pattern::NonAzimuthDependent(values.clone()))
        //        } else {
        //            let angle = f64::from_str(content.trim()).unwrap();
        //            frequency =
        //                frequency.add_pattern(Pattern::AzimuthDependent((angle, values.clone())))
        //        }
        //    }
    }

    //Ok((antenna, frequencies))
    panic!("not yet")
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_epoch() {
        let content = "                                                           START OF ANTENNA";
        assert!(is_new_epoch(content));
        let content =
            "TROSAR25.R4      LEIT727259                                 TYPE / SERIAL NO";
        assert!(!is_new_epoch(content));
        let content =
            "    26                                                      # OF FREQUENCIES";
        assert!(!is_new_epoch(content));
        let content =
            "   G01                                                      START OF FREQUENCY";
        assert!(!is_new_epoch(content));
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
        //for antenna in rhs.iter() {
        //    if self.contains(antenna) {
        //        let (antenna, frequencies) = antenna;
        //        for (aantenna, ffrequencies) in self.iter_mut() {
        //            if antenna == aantenna {
        //                // for this antenna
        //                // add missing frequencies
        //                for frequency in frequencies {
        //                    if !ffrequencies.contains(frequency) {
        //                        ffrequencies.push(frequency.clone());
        //                    }
        //                }
        //            }
        //        }
        //    } else {
        //        self.push(antenna.clone());
        //    }
        //}
        Ok(())
    }
}
