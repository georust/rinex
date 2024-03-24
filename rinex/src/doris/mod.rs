use std::collections::HashMap;
use thiserror::Error;

use crate::{
    domes::{Domes, Error as DomesParsingError, TrackingPoint as DomesTrackingPoint},
    observable::Observable,
};
use hifitime::Epoch;

pub(crate) mod record;

pub use record::Record;

/// DORIS Station & record parsing error
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid station")]
    InvalidStation,
    #[error("invalid station DOMES code")]
    DomesError(#[from] DomesParsingError),
    #[error("failed to parse station value")]
    StationIdOrValueParsing,
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

/*
 * Parses DORIS station, returns ID# code and Station
 */
pub(crate) fn parse_station(content: &str) -> Result<Station, Error> {
    if content.len() < 40 {
        return Err(Error::InvalidStation);
    }

    let content = content.split_at(1).1;
    let (key, rem) = content.split_at(5);
    let (label, rem) = rem.split_at(5);
    let (name, rem) = rem.split_at(30);
    let (domes, rem) = rem.split_at(10);
    let (gen, rem) = rem.split_at(3);
    let (k_factor, _) = rem.split_at(3);

    println!("ID \"{}\"", key);
    println!("LABEL \"{}\"", label);
    println!("NAME \"{}\"", name);
    println!("DOMES \"{}\"", domes);
    println!("GEN \"{}\"", gen);
    println!("K \"{}\"", k_factor);

    panic!("oops");
}

impl std::fmt::Display for Station {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "D{:02} {} {:<20} {} {} {}",
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
        for (desc, expected) in [(
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
        )] {
            let station = parse_station(desc).unwrap();
            assert_eq!(station, expected, "station parsing error");
            // reciprocal
            assert_eq!(station.to_string(), desc);
        }
    }
}
