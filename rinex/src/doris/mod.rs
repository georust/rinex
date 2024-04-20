use std::collections::HashMap;

use thiserror::Error;

use crate::{
    domes::Error as DomesParsingError,
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
