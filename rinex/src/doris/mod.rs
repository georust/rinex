use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

use crate::{
    domes::{Domes, Error as DomesParsingError, TrackingPoint as DomesTrackingPoint},
    observable::Observable,
    prelude::{Duration, Epoch},
};

pub(crate) mod record;

pub use record::Record;

/// DORIS Station & record parsing error
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid station")]
    InvalidStation,
    #[error("failed to parse station id")]
    IdParsing,
    #[error("invalid station DOMES code")]
    DomesError(#[from] DomesParsingError),
    #[error("failed to parse beacon generation")]
    BeaconGenerationParsing,
    #[error("failed to parse `k` factor")]
    KfParsing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Station {
    /// Station mnemonic label (Antenna point)
    pub label: String,
    /// DORIS site name
    pub site: String,
    /// DOMES site identifier
    pub domes: Domes,
    /// Beacon generation
    pub gen: u8,
    /// K frequency shift factor
    pub k_factor: i8,
    /// ID used in this file indexing
    pub(crate) key: u16,
}

impl Station {
    const USO_FREQ: f64 = 5.0E6_f64;
    /// Station S1 Frequency shift factor
    pub fn s1_frequency_shift(&self) -> f64 {
        543.0 * Self::USO_FREQ * (3.0 / 4.0 + 87.0 * self.k_factor as f64 / 5.0 * 2.0_f64.powi(26))
    }
    /// Station U2 Frequency shift factor
    pub fn u2_frequency_shift(&self) -> f64 {
        107.0 * Self::USO_FREQ * (3.0 / 4.0 + 87.0 * self.k_factor as f64 / 5.0 * 2.0_f64.powi(26))
    }
}

/*
 * Parses DORIS station, returns ID# code and Station
 */
pub(crate) fn parse_station(content: &str) -> Result<Station, Error> {
    if content.len() < 40 {
        return Err(Error::InvalidStation);
    }

    let content = content.split_at(1).1;
    let (key, rem) = content.split_at(4);
    let (label, rem) = rem.split_at(5);
    let (name, rem) = rem.split_at(30);
    let (domes, rem) = rem.split_at(10);
    let (gen, rem) = rem.split_at(3);
    let (k_factor, _) = rem.split_at(3);

    Ok(Station {
        site: name.trim().to_string(),
        label: label.trim().to_string(),
        domes: Domes::from_str(domes.trim())?,
        gen: gen
            .trim()
            .parse::<u8>()
            .or(Err(Error::BeaconGenerationParsing))?,
        k_factor: k_factor.trim().parse::<i8>().or(Err(Error::KfParsing))?,
        key: key.trim().parse::<u16>().or(Err(Error::IdParsing))?,
    })
}

impl std::fmt::Display for Station {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "D{:02}  {} {:<29} {}  {}   {}",
            self.key, self.label, self.site, self.domes, self.gen, self.k_factor
        )
    }
}

/// DORIS Record specific header fields
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Time of First Measurement, expressed in TAI timescale.
    pub time_of_first_obs: Option<Epoch>,
    /// Time of Last Measurement, expressed in TAI timescale.
    pub time_of_last_obs: Option<Epoch>,
    /// List of observables
    pub observables: Vec<Observable>,
    /// Data scaling, almost 100% of the time present in DORIS measurements.
    /// Allows some nano radians precision on phase data for example.
    pub scaling: HashMap<Observable, u16>,
    /// Reference stations present in this file
    pub stations: Vec<Station>,
    /// Constant shift between date of the 400 MHz phase measurement
    /// and date of the 2GHz phase measurement
    pub l2_l1_date_offset: Duration,
}

impl HeaderFields {
    /// Add TIME OF FIRST OBS
    pub(crate) fn with_time_of_first_obs(&self, epoch: Epoch) -> Self {
        let mut s = self.clone();
        s.time_of_first_obs = Some(epoch);
        s
    }
    /// Add TIME OF LAST OBS
    pub(crate) fn with_time_of_last_obs(&self, epoch: Epoch) -> Self {
        let mut s = self.clone();
        s.time_of_last_obs = Some(epoch);
        s
    }
    /// Add L2 /L1 Date offset
    pub(crate) fn with_l2_l1_date_offset(&self, offset: Duration) -> Self {
        let mut s = self.clone();
        s.l2_l1_date_offset = offset;
        s
    }
    /// Define station #ID
    pub(crate) fn add_station(&mut self, station: Station) {
        self.stations.push(station);
    }
    /// Retrieve station by ID#
    pub(crate) fn get_station(&mut self, id: u16) -> Option<&Station> {
        self.stations
            .iter()
            .filter(|s| s.key == id)
            .reduce(|k, _| k)
    }
    /// Insert a data scaling
    pub(crate) fn with_scaling(&mut self, observable: Observable, scaling: u16) {
        self.scaling.insert(observable.clone(), scaling);
    }
    /// Returns scaling to applied to said Observable.
    pub(crate) fn scaling(&self, observable: Observable) -> Option<&u16> {
        self.scaling.get(&observable)
    }
}

#[cfg(test)]
mod test {
    use super::{parse_station, Station};
    use crate::domes::{Domes, TrackingPoint as DomesTrackingPoint};
    #[test]
    fn station_parsing() {
        for (desc, expected) in [
            (
                "D01  OWFC OWENGA                        50253S002  3   0",
                Station {
                    label: "OWFC".to_string(),
                    site: "OWENGA".to_string(),
                    domes: Domes {
                        area: 502,
                        site: 53,
                        sequential: 2,
                        point: DomesTrackingPoint::Instrument,
                    },
                    gen: 3,
                    k_factor: 0,
                    key: 1,
                },
            ),
            (
                "D17  GRFB GREENBELT                     40451S178  3   0",
                Station {
                    label: "GRFB".to_string(),
                    site: "GREENBELT".to_string(),
                    domes: Domes {
                        area: 404,
                        site: 51,
                        sequential: 178,
                        point: DomesTrackingPoint::Instrument,
                    },
                    gen: 3,
                    k_factor: 0,
                    key: 17,
                },
            ),
        ] {
            let station = parse_station(desc).unwrap();
            assert_eq!(station, expected, "station parsing error");
            assert_eq!(station.to_string(), desc, "station reciprocal error");
        }
    }
}
