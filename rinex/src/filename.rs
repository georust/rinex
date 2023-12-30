/*
 * Some attributes can only be determined from the name of the file,
 * like the country code.
 * The filename is based off the geodetic marker name.
 * Meteo RINEX and NAV RINEX usually do not have information about the geodetic marker.
 * Missing FilenameAttributes is not important, it just means we will not be able
 * to fully reconstruct a file that follows standard naming conventions.
 */
use hifitime::Epoch;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {}

/// Data Source as described in standard filenames.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum DataSource {
    /// Source of data is hardware (radio) receiver.
    /// It can also represent a sensor in case of meteo observations.
    Receiver,
    /// Other stream source, like RTCM
    Stream,
    /// Unknown data source
    #[default]
    Unknown,
}

impl std::str::FromStr for DataSource {
    type Err = std::io::Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        if content.eq("R") {
            Ok(Self::Receiver)
        } else if content.eq("S") {
            Ok(Self::Stream)
        } else {
            Ok(Self::Unknown)
        }
    }
}

/// Filename Attributes from standardized modern (V3+) file names.
#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct FilenameAttributes {
    /// Station name (complete). This is usually named
    /// after a geodetic marker.
    pub station: String,
    /// Data source
    pub data_src: DataSource,
    /// Epoch of file production
    pub production_epoch: Epoch,
    /// 3 letter country code
    pub country: String,
}

impl FilenameAttributes {}

impl std::str::FromStr for FilenameAttributes {
    type Err = Error;
    fn from_str(fname: &str) -> Result<Self, Self::Err> {
        Ok(Self::default())
    }
}

#[cfg(test)]
fn test_from_str() {
    let attrs = FilenameAttributes::from_str("");
    assert!(attrs.is_ok());
}
