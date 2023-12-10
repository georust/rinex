use gnss::prelude::SV;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

use super::{
    antenna::SvAntennaParsingError, Antenna, AntennaSpecific, Calibration, CalibrationMethod,
    Cospar, RxAntenna, SvAntenna,
};
use crate::{carrier, linspace::Linspace, merge, merge::Merge, Carrier, Epoch};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Returns true if this line matches
/// the beginning of a `epoch` for ATX file (special files),
/// this is not really an epoch but rather a group of dataset
/// for this given antenna, there is no sampling data attached to it.
pub(crate) fn is_new_epoch(content: &str) -> bool {
    content.contains("START OF ANTENNA")
}

/// Phase pattern description.
/// We currently do not support azimuth dependent phase patterns.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum AntennaPhasePattern {
    /// Azimuth Independent Phase pattern
    AzimuthIndependentPattern(Vec<f64>),
}

impl Default for AntennaPhasePattern {
    fn default() -> Self {
        Self::AzimuthIndependentPattern(Vec::<f64>::new())
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
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
pub type Record = Vec<(Antenna, HashMap<Carrier, FrequencyDependentData>)>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown PCV \"{0}\"")]
    UnknownPcv(String),
    #[error("failed to determine the frequency")]
    SvParsing(#[from] gnss::sv::ParsingError),
    #[error("failed to determine the frequency")]
    ParseCarrierError(#[from] carrier::Error),
    #[error("sv antenna parsing error")]
    SvAntennaParsing(#[from] SvAntennaParsingError),
    #[error("failed to parse APC/NEU northern coordinates")]
    APCNorthernCoordinatesParsing,
    #[error("failed to parse APC/NEU eastern coordinates")]
    APCEasternCoordinatesParsing,
    #[error("failed to parse APC/NEU upper coordinates")]
    APCUpperCoordinatesParsing,
    #[error("failed to identify number of calibrated antennas")]
    NumberOfCalibratedAntennasParsing,
    #[error("failed to parse start of validity period")]
    StartOfValidityPeriodParsing,
    #[error("failed to parse end of validity period")]
    EndOfValidityPeriodParsing,
    #[error("failed to parse year of this calibration")]
    DatetimeYearParsing,
    #[error("failed to parse month of this calibration")]
    DatetimeMonthParsing,
    #[error("failed to parse day of this calibration")]
    DatetimeDayParsing,
    #[error("failed to parse start of zenith grid")]
    ZenithGridStartParsing,
    #[error("failed to parse end of zenith grid")]
    ZenithGridEndParsing,
    #[error("failed to parse spacing of zenith grid")]
    ZenithGridSpacingParsing,
}

fn parse_datetime(content: &str) -> Result<Epoch, Error> {
    let mut parser = content.split('-');

    let year = parser.next().ok_or(Error::DatetimeYearParsing)?;
    let year = year
        .parse::<i32>()
        .map_err(|_| Error::DatetimeYearParsing)?;

    let month = parser.next().ok_or(Error::DatetimeMonthParsing)?;

    let month = match month {
        "JAN" => 1,
        "FEB" => 2,
        "MAR" => 3,
        "APR" => 4,
        "MAY" => 5,
        "JUN" => 6,
        "JUL" => 7,
        "AUG" => 8,
        "SEP" => 9,
        "OCT" => 10,
        "NOV" => 11,
        "DEC" => 12,
        _ => {
            return Err(Error::DatetimeMonthParsing);
        },
    };

    let day = parser.next().ok_or(Error::DatetimeDayParsing)?;
    let day = day.parse::<u8>().map_err(|_| Error::DatetimeDayParsing)?;

    Ok(Epoch::from_gregorian_utc_at_midnight(
        2000 + year,
        month,
        day,
    ))
}

/// Parses entire Antenna block
/// and all inner frequency entries
pub(crate) fn parse_antenna(
    content: &str,
) -> Result<(Antenna, HashMap<Carrier, FrequencyDependentData>), Error> {
    let lines = content.lines();
    let mut antenna = Antenna::default();
    let mut inner = HashMap::<Carrier, FrequencyDependentData>::new();
    let mut frequency = Carrier::default();
    let mut freq_data = FrequencyDependentData::default();
    let mut valid_from = Epoch::default();

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
            let (number, rem) = rem.split_at(10); // N#
            let (date, _) = rem.split_at(10);

            let cal = Calibration {
                method: CalibrationMethod::from_str(method.trim()).unwrap(),
                number: number
                    .trim()
                    .parse::<u16>()
                    .map_err(|_| Error::NumberOfCalibratedAntennasParsing)?,
                agency: agency.trim().to_string(),
                date: parse_datetime(date.trim())?,
            };

            antenna.calibration = cal.clone();
        } else if marker.contains("VALID FROM") {
            valid_from =
                Epoch::from_str(content.trim()).map_err(|_| Error::StartOfValidityPeriodParsing)?;
        } else if marker.contains("VALID UNTIL") {
            let valid_until =
                Epoch::from_str(content.trim()).map_err(|_| Error::EndOfValidityPeriodParsing)?;

            antenna = antenna.with_validity_period(valid_from, valid_until);
        } else if marker.contains("SINEX CODE") {
            let sinex = content.split_at(20).0;
            antenna.sinex_code = sinex.trim().to_string();
        } else if marker.contains("DAZI") {
            //let dazi = content.split_at(20).0.trim();
            //if let Ok(dazi) = f64::from_str(dazi) {
            //    antenna = antenna.with_dazi(dazi)
            //}
        } else if marker.contains("# OF FREQUENCIES") {
            /*
             * we actually do not care about this field
             * it is easy to determine it from the current infrastructure
             */
        } else if marker.contains("START OF FREQUENCY") {
            let svnn = content.split_at(10).0;
            let sv = SV::from_str(svnn.trim())?;
            frequency = carrier::Carrier::from_sv(sv)?;
        } else if marker.contains("NORTH / EAST / UP") {
            let (north, rem) = content.split_at(10);
            let (east, rem) = rem.split_at(10);
            let (up, _) = rem.split_at(10);
            let north = north
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::APCNorthernCoordinatesParsing)?;
            let east = east
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::APCEasternCoordinatesParsing)?;
            let up = up
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::APCUpperCoordinatesParsing)?;

            freq_data.apc_eccentricity = (north, east, up);
        } else if marker.contains("ZEN1 / ZEN2 / DZEN") {
            let (start, rem) = content.split_at(8);
            let (end, rem) = rem.split_at(6);
            let (spacing, _) = rem.split_at(6);

            let start = start
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::ZenithGridStartParsing)?;
            let end = end
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::ZenithGridEndParsing)?;
            let spacing = spacing
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::ZenithGridSpacingParsing)?;

            antenna.zenith_grid = Linspace {
                start,
                end,
                spacing,
            };
        } else if marker.contains("END OF FREQUENCY") {
            inner.insert(frequency, freq_data.clone());
        } else if marker.contains("END OF ANTENNA") {
            break; // end of this block, considered as an `epoch`
                   // if we make a parallel with other types of RINEX
        } else {
            // inside phase pattern
        }
        //    } else if marker.contains("SINEX CODE") {
        //        let sinex = content.split_at(10).0;
        //        antenna = antenna.with_sinex_code(sinex.trim())
        //        if line.contains("NOAZI") {
        //            frequency = frequency.add_pattern(Pattern::NonAzimuthDependent(values.clone()))
        //        } else {
        //            let angle = f64::from_str(content.trim()).unwrap();
        //            frequency =
        //                frequency.add_pattern(Pattern::AzimuthDependent((angle, values.clone())))
        //        }
        //    }
    }

    Ok((antenna, inner))
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
