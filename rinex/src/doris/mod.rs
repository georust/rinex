use std::collections::HashMap;
use thiserror::Error;

use crate::observable::Observable;
use hifitime::Epoch;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Station {
    /// Station mnemonic label (Antenna point)
    pub label: String,
    /// DORIS site name
    pub site: String,
    /// DOMES number
    pub domes: String, // TODO: use DOMES struct
    /// Beacon generation
    pub gen: u8,
    /// K frequency shift factor
    pub k_factor: i8,
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
    /// ID# given to each station that will later serve as Record index key
    pub(crate) stations: HashMap<Station, u16>,
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
    pub(crate) fn set_station_id(&mut self, station: Station, id: u16) {
        self.stations.insert(station, id);
    }
    /// Retrieve station ID#
    pub(crate) fn get_station_id(&mut self, station: &Station) -> Option<&u16> {
        self.stations.get(station)
    }
    /// Retrieve station by ID#
    pub(crate) fn get_station(&mut self, id: u16) -> Option<&Station> {
        self.stations
            .iter()
            .filter_map(|(k, v)| if *v == id { Some(k) } else { None })
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

#[derive(Debug, Error)]
pub enum Error {}

pub(crate) mod record;

pub use record::Record;
