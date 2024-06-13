use std::collections::HashMap;
use thiserror::Error;

use crate::{
    observable::Observable,
    prelude::{Duration, Epoch},
};

use gnss_rs::domes::Error as DomesParsingError;

pub(crate) mod record;
pub(crate) mod station;

pub use record::Record;
pub use station::Station;

#[cfg(feature = "processing")]
use crate::prelude::TimeScale;

#[cfg(feature = "processing")]
use itertools::Itertools;

#[cfg(feature = "processing")]
use qc_traits::processing::{FilterItem, MaskFilter, MaskOperand};

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
    /// Name of the DORIS satellite
    pub satellite: String,
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
    // /// Retrieve station by ID#
    // pub(crate) fn get_station(&mut self, id: u16) -> Option<&Station> {
    //     self.stations
    //         .iter()
    //         .filter(|s| s.key == id)
    //         .reduce(|k, _| k)
    // }
    /// Insert a data scaling
    pub(crate) fn with_scaling(&mut self, observable: Observable, scaling: u16) {
        self.scaling.insert(observable.clone(), scaling);
    }
    // /// Returns scaling to applied to said Observable.
    // pub(crate) fn scaling(&self, observable: Observable) -> Option<&u16> {
    //     self.scaling.get(&observable)
    // }
}

#[cfg(feature = "processing")]
impl HeaderFields {
    fn timescale(&self) -> TimeScale {
        match self.time_of_first_obs {
            Some(ts) => ts.time_scale,
            None => match self.time_of_last_obs {
                Some(ts) => ts.time_scale,
                None => TimeScale::GPST,
            },
        }
    }
    pub(crate) fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::Equals => match &f.item {},
            MaskOperand::NotEquals => match &f.item {},
            MaskOperand::GreaterThan => match &f.item {},
            MaskOperand::GreaterEquals => match &f.item {},
            MaskOperand::LowerThan => match &f.item {},
            MaskOperand::LowerEquals => match &f.item {},
        }
    }
}
