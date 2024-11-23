use crate::{
    antex,
    clock::{self, ClockKey, ClockProfile},
    doris,
    hatanaka::DecompressorExpert,
    header, ionex, is_rinex_comment, meteo,
    navigation::{self, record::parse_epoch as parse_nav_epoch},
    observation::{
        is_new_epoch as is_new_observation_epoch, parse_epoch as parse_observation_epoch,
        Record as ObservationRecord,
    },
    prelude::{Constellation, Epoch, Header, Observations, ParsingError, TimeScale},
    types::Type,
};

use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Read},
    str::from_utf8,
};

#[cfg(feature = "log")]
use log::error;

#[cfg(feature = "serde")]
use serde::Serialize;

mod formatting;
mod parsing;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Record {
    /// ATX record, see [antex::record::Record]
    AntexRecord(antex::Record),
    /// Clock record, see [clock::record::Record]
    ClockRecord(clock::Record),
    /// IONEX (Ionosphere maps) record, see [ionex::record::Record]
    IonexRecord(ionex::Record),
    /// Meteo record, see [meteo::record::Record]
    MeteoRecord(meteo::Record),
    /// Navigation record, see [navigation::record::Record]
    NavRecord(navigation::Record),
    /// Observation record [ObservationRecord]
    ObsRecord(ObservationRecord),
    /// DORIS RINEX, special DORIS measurements wraped as observations
    DorisRecord(doris::Record),
}

/// Record comments are high level informations, sorted by epoch
/// (timestamp) of appearance. We deduce the "associated" timestamp from the
/// previosuly parsed epoch, when parsing the record.
pub type Comments = BTreeMap<Epoch, Vec<String>>;

impl Record {
    /// Unwraps self as ANTEX record
    pub fn as_antex(&self) -> Option<&antex::Record> {
        match self {
            Record::AntexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable reference to ANTEX record
    pub fn as_mut_antex(&mut self) -> Option<&mut antex::Record> {
        match self {
            Record::AntexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as CLK record
    pub fn as_clock(&self) -> Option<&clock::Record> {
        match self {
            Record::ClockRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable CLK record
    pub fn as_mut_clock(&mut self) -> Option<&mut clock::Record> {
        match self {
            Record::ClockRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as IONEX record
    pub fn as_ionex(&self) -> Option<&ionex::Record> {
        match self {
            Record::IonexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable IONEX record
    pub fn as_mut_ionex(&mut self) -> Option<&mut ionex::Record> {
        match self {
            Record::IonexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as MET record
    pub fn as_meteo(&self) -> Option<&meteo::Record> {
        match self {
            Record::MeteoRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Returns mutable reference to Meteo record
    pub fn as_mut_meteo(&mut self) -> Option<&mut meteo::Record> {
        match self {
            Record::MeteoRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as NAV record
    pub fn as_nav(&self) -> Option<&navigation::Record> {
        match self {
            Record::NavRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Returns mutable reference to Navigation record
    pub fn as_mut_nav(&mut self) -> Option<&mut navigation::Record> {
        match self {
            Record::NavRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as OBS record
    pub fn as_obs(&self) -> Option<&ObservationRecord> {
        match self {
            Record::ObsRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Returns mutable reference to Observation record
    pub fn as_mut_obs(&mut self) -> Option<&mut ObservationRecord> {
        match self {
            Record::ObsRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as DORIS record
    pub fn as_doris(&self) -> Option<&doris::Record> {
        match self {
            Record::DorisRecord(r) => Some(r),
            _ => None,
        }
    }

    /// Unwraps self as mutable reference to DORIS record
    pub fn as_mut_doris(&mut self) -> Option<&mut doris::Record> {
        match self {
            Record::DorisRecord(r) => Some(r),
            _ => None,
        }
    }
}
