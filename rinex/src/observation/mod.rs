//! Observation RINEX module
use thiserror::Error;

use std::{
    collections::HashMap,
    str::FromStr,
};

use crate::{
    prelude::{Epoch, SV},
    version::Version,
    observable::Observable,
    epoch::parse_utc as parse_utc_epoch,
};

mod snr;
mod crinex;
mod lli;
mod observation;

mod parser; // parse_* helpers
mod formater; // fmt_* helpers

mod merge; // [Record] merge ops
mod split; // [Record] split ops
mod processing; // [Processing] specific ops

pub mod flag;
pub mod header;

pub use snr::SNR;
pub use lli::LliFlags;
pub use flag::EpochFlag;
pub use crinex::Crinex;
pub use header::HeaderFields;
pub use observation::{ObservationEntry, ObservationEvent, SignalObservation};

#[cfg(docsrs)]
use crate::Bibliography;

#[cfg(feature = "processing")]
use crate::prelude::TimeScale;

#[cfg(feature = "processing")]
use itertools::Itertools;

#[cfg(feature = "obs")]
use std::collections::BTreeMap;

use crate::{
    observation::flag::Error as FlagError,
    epoch::ParsingError as EpochParsingError,
};

use gnss_rs::{
    constellation::ParsingError as ConstellationParsingError,
    sv::ParsingError as SVParsingError,
};

/// Observation related [Error]s
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse epoch flag")]
    EpochFlag(#[from] FlagError),
    #[error("failed to parse epoch")]
    EpochError(#[from] EpochParsingError),
    #[error("constellation parsing error")]
    ConstellationParsing(#[from] ConstellationParsingError),
    #[error("sv parsing error")]
    SVParsing(#[from] SVParsingError),
    // #[error("failed to parse integer measurement")]
    // ParseIntError(#[from] std::num::ParseIntError),
    // #[error("failed to parse float number")]
    // ParseFloatError(#[from] std::num::ParseFloatError),
    // #[error("failed to parse vehicles properly (nb_sat mismatch)")]
    // EpochParsingError,
    // #[error("line is empty")]
    // MissingData,
    #[error("missing observable definitions")]
    MissingObservableSpecs,
}


#[cfg(feature = "obs")]
#[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
#[derive(Debug, Copy, Clone)]
pub enum Combination {
    GeometryFree,
    IonosphereFree,
    WideLane,
    NarrowLane,
    MelbourneWubbena,
}

/// GNSS signal combination trait.    
/// This only applies to OBS RINEX records.  
/// Refer to [Bibliography::ESAGnssCombination] and [Bibliography::ESABookVol1]
/// for more information.
#[cfg(feature = "obs")]
#[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
pub trait Combine {
    fn combine(
        &self,
        combination: Combination,
    ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>;
}

/// GNSS code bias estimation trait.
/// Refer to [Bibliography::ESAGnssCombination] and [Bibliography::ESABookVol1].
#[cfg(feature = "obs")]
#[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
pub trait Dcb {
    /// Returns Differential Code Bias estimates, sorted per (unique)
    /// signals combinations and for each individual SV.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observation::*; // .dcb()
    ///
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///    .unwrap();
    /// let dcb = rinex.dcb();
    /// ```
    fn dcb(&self) -> HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>;
}

/// Macro used in new text epoch determination (while parsing)
pub(crate) fn is_new_epoch(line: &str, v: Version) -> bool {
    if v.major < 3 {
        if line.len() < 30 {
            false
        } else {
            // SPLICE flag handling (still an Observation::flag)
            let significant = !line[0..26].trim().is_empty();
            let epoch = parse_utc_epoch(&line[0..26]);
            let flag = EpochFlag::from_str(line[26..29].trim());
            if significant {
                epoch.is_ok() && flag.is_ok()
            } else if flag.is_err() {
                false
            } else {
                match flag.unwrap() {
                    EpochFlag::AntennaBeingMoved
                    | EpochFlag::NewSiteOccupation
                    | EpochFlag::HeaderInformationFollows
                    | EpochFlag::ExternalEvent => true,
                    _ => false,
                }
            }
        }
    } else {
        // Modern RINEX has a simple marker, like all V4 modern files
        match line.chars().next() {
            Some(c) => {
                c == '>' // epochs always delimited
                         // by this new identifier
            },
            _ => false,
        }
    }
}

mod test {
    use super::*;
    #[test]
    fn obs_record_is_new_epoch() {
        assert!(is_new_epoch(
            "95 01 01 00 00 00.0000000  0  7 06 17 21 22 23 28 31",
            Version { major: 2, minor: 0 }
        ));
        assert!(!is_new_epoch(
            "21700656.31447  16909599.97044          .00041  24479973.67844  24479975.23247",
            Version { major: 2, minor: 0 }
        ));
        assert!(is_new_epoch(
            "95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
            Version { major: 2, minor: 0 }
        ));
        assert!(!is_new_epoch(
            "95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
            Version { major: 3, minor: 0 }
        ));
        assert!(is_new_epoch(
            "> 2022 01 09 00 00 30.0000000  0 40",
            Version { major: 3, minor: 0 }
        ));
        assert!(!is_new_epoch(
            "> 2022 01 09 00 00 30.0000000  0 40",
            Version { major: 2, minor: 0 }
        ));
        assert!(!is_new_epoch(
            "G01  22331467.880   117352685.28208        48.950    22331469.28",
            Version { major: 3, minor: 0 }
        ));
    }
}