use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

use crate::{
    domes::{Domes, Error as DomesParsingError, TrackingPoint as DomesTrackingPoint},
    observable::Observable,
    prelude::{Duration, Epoch},
};

pub(crate) mod record;
pub(crate) mod station;

pub use record::Record;
pub use station::Station;

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
    /// Constant shift between date of the U2 (401.25 MHz) phase measurement
    /// and date of the S1 (2.03625 GHz) phase measurement
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
