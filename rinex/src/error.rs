use gnss_rs::{
    constellation::ParsingError as ConstellationParsingError, cospar::error as CosparParsingError,
};

use crate::{clock::Error as ClockError, navigation::Error as NavigationError};

/// Errors that may rise in Parsing process
#[derive(Error, Debug)]
pub enum ParsingError {
    /// Valid RINEX Header lines should be 60 byte long
    HeaderLineTooShort,
    /// Error when parsing Date/Time in general
    DateTimeParsing,
    /// Invalid VERSION Header
    VersionParsing,
    /// Invalid or Type not recognized (= RINEX TYPE)
    TypeParsing,
    /// Invalid or non supported Observable
    ObservableParsing,
    /// Constellation Parsing Error
    ConstellationParsing(#[from] ConstellationParsingError),
    /// COSPAR Parsing Error (invalid)
    COSPAR(#[from] CosparParsingError),
    /// Navigation specific Parsing error
    NavigationParsing(#[from] NavigationError),
    /// Clock specific Parsing error
    ClockParsing(#[from] ClockError),
    /// Invalid Observation RINEX: Header misses
    /// Time of First/Last fields: impossible to interprate following content
    BadObsNoTimescaleDefinition,
}

/// Errors that may rise in Formatting process
#[derive(Error, Debug)]
pub enum FormattingError {}
