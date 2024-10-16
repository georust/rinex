use thiserror::Error;

use gnss_rs::{
    constellation::ParsingError as ConstellationParsingError, cospar::Error as CosparParsingError,
    sv::ParsingError as SVParsingError,
};

use crate::{clock::Error as ClockError, navigation::Error as NavigationError};

use hifitime::ParsingError as HifitimeParsingError;

use std::io::Error as IoError;

/// Errors that may rise in Parsing process
#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("header line too short (invalid)")]
    HeaderLineTooShort,
    #[error("datime parsing error")]
    DatetimeParsing,
    #[error("datime invalid format")]
    DatetimeFormat,
    #[error("bad rinex revision format")]
    VersionFormat,
    #[error("rinex revision parsing error")]
    VersionParsing,
    #[error("rinex format identification error")]
    TypeParsing,
    #[error("observable parsing error")]
    ObservableParsing,
    #[error("constellation parsing error")]
    ConstellationParsing(#[from] ConstellationParsingError),
    #[error("sv parsing error")]
    SVParsing(#[from] SVParsingError),
    #[error("cospar parsing error")]
    COSPAR(#[from] CosparParsingError),
    #[error("navdata parsing error")]
    NavigationParsing(#[from] NavigationError),
    /// Clock specific Parsing error
    #[error("clkdata parsing error")]
    ClockData(#[from] ClockError),
    #[error("clock TYPE OF DATA parsing error")]
    ClockTypeofData,
    #[error("OBS RINEX invalid timescale")]
    BadObsBadTimescaleDefinition,
    #[error("OBS RINEX parsing requires proper timescale specs")]
    BadObsNoTimescaleDefinition,
    #[error("SYS / SCALE FACTOR parsing")]
    SystemScalingFactor,
    #[error("REF CLOCK OFFS parsing")]
    RcvClockOffsApplied,
    #[error("header coordinates parsing")]
    Coordinates,
    #[error("antenna coordinates parsing")]
    AntennaCoordinates,
    #[error("sensor coordinates parsing")]
    SensorCoordinates,
    #[error("ANTEX version parsing")]
    AntexVersion,
    #[error("IONEX version parsing")]
    IonexVersion,
    #[error("invalid ionex referencing")]
    IonexReferenceSystem,
    #[error("non supported rinex revision")]
    NonSupportedVersion,
    #[error("unknown/non supported observable")]
    UnknownObservable,
    #[error("invalid observable")]
    BadObservable,
    #[error("invalid leap second specs")]
    LeapFormat,
    #[error("leap second parsing")]
    LeapParsing,
    #[error("hifitime parsing")]
    HifitimeParsing(#[from] HifitimeParsingError),
    #[error("DORIS L1/L2 date offset")]
    DorisL1L2DateOffset,
    #[error("DORIS L1/L2 date offset")]
    NumberOfCalibratedAntennasParsing,
    #[error("antex: antenna calibration #")]
    AntexAntennaCalibrationNumber,
    #[error("antex: apc coordinates")]
    AntexAPCCoordinates,
    #[error("antex: zenith grid")]
    AntexZenithGrid,
    #[error("antex: frequency")]
    AntexFrequency,
}

/// Errors that may rise in Formatting process
#[derive(Error, Debug)]
pub enum FormattingError {
    #[error("i/o: output error")]
    OutputError(#[from] IoError),
}

/// General error (processing, analysis..)
#[derive(Debug)]
pub enum Error {
    /// Non supported GPS [Observable]
    UnknownGPSObservable,
    /// Non supported Galileo [Observable]
    UnknownGalieoObservable,
    /// Non supported Glonass [Observable]
    UnknownGlonassObservable,
    /// Non supported QZSS [Observable]
    UnknownQZSSObservable,
    /// Non supported BDS [Observable]
    UnknownBeiDouObservable,
    /// Non supported IRNSS [Observable]
    UnknownIRNSSObservable,
    /// Non supported SBAS [Observable]
    UnknownSBASObservable,
    /// Non supported DORIS [Observable]
    UnknownDORISObservable,
}
