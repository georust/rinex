/*
 * Some attributes can only be determined from the name of the file,
 * like the country code.
 * The filename is based off the geodetic marker name.
 * Meteo RINEX and NAV RINEX usually do not have information about the geodetic marker.
 * Missing FilenameAttributes is not important, it just means we will not be able
 * to fully reconstruct a file that follows standard naming conventions.
 */
use hifitime::Epoch;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Filename does not follow naming conventions")]
    NonStandardFilename,
}

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
        if fname.len() < 34 {
            return Err(Error::NonStandardFilename);
        }
        if fname.chars().nth(9) != Some('_') {
            return Err(Error::NonStandardFilename);
        }
        if fname.chars().nth(11) != Some('_') {
            return Err(Error::NonStandardFilename);
        }
        if fname.chars().nth(23) != Some('_') {
            return Err(Error::NonStandardFilename);
        }
        Ok(Self {
            station: fname[..6].trim().to_string(),
            country: fname[6..9].trim().to_string(),
            // data_src parsing is infaillible
            data_src: DataSource::from_str(&fname[10..11]).unwrap(),
            production_epoch: Epoch::default(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::DataSource;
    use super::FilenameAttributes;
    use std::str::FromStr;
    #[test]
    fn file_attributes() {
        for (filename, station, country, data_src) in [
            (
                "ACOR00ESP_R_20213550000_01D_30S_MO",
                "ACOR00",
                "ESP",
                DataSource::Receiver,
            ),
            (
                "ESBC00DNK_R_20201770000_01D_30S_MO",
                "ESBC00",
                "DNK",
                DataSource::Receiver,
            ),
        ] {
            let attrs = FilenameAttributes::from_str(filename);
            assert!(attrs.is_ok());
            let attrs = attrs.unwrap();
            assert_eq!(attrs.country, country);
            assert_eq!(attrs.station, station);
            assert_eq!(attrs.data_src, data_src);
        }
    }
}
