use crate::context::Error;
use rinex::types::Type as RinexType;
use std::path::Path;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProductType {
    /// Non Supported or unknown product
    #[default]
    Undefined,
    /// GNSS signal observations provided by Observation RINEX.
    Observation,
    /// Meteo observations provided by Meteo RINEX.
    MeteoObservation,
    /// DORIS signals observation provided by special RINEX.
    DORIS,
    /// Broadcast Navigation message described by Navigation RINEX.
    BroadcastNavigation,
    /// High precision clock states described by Clock RINEX.
    HighPrecisionClock,
    /// Antenna calibration information described by ANTEX.
    ANTEX,
    /// Precise Ionosphere maps described by IONEX.
    IONEX,
    #[cfg(feature = "sp3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
    /// High precision orbital attitude described by SP3.
    HighPrecisionOrbit,
}

impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Undefined => write!(f, "Undefined"),
            Self::ANTEX => write!(f, "ANTEX"),
            Self::IONEX => write!(f, "IONEX"),
            Self::DORIS => write!(f, "DORIS"),
            Self::Observation => write!(f, "Observation RINEX"),
            Self::MeteoObservation => write!(f, "Meteo RINEX"),
            Self::HighPrecisionClock => write!(f, "Clock RINEX"),
            Self::BroadcastNavigation => write!(f, "Navigation RINEX"),
            #[cfg(feature = "sp3")]
            Self::HighPrecisionOrbit => write!(f, "SP3"),
        }
    }
}

impl std::fmt::LowerHex for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ANTEX => write!(f, "ANTEX"),
            Self::IONEX => write!(f, "IONEX"),
            Self::DORIS => write!(f, "DORIS"),
            Self::Observation => write!(f, "OBS"),
            Self::Undefined => write!(f, "UNDEFINED"),
            Self::MeteoObservation => write!(f, "METEO"),
            Self::HighPrecisionClock => write!(f, "CLOCK"),
            Self::BroadcastNavigation => write!(f, "BRDC"),
            #[cfg(feature = "sp3")]
            Self::HighPrecisionOrbit => write!(f, "SP3"),
        }
    }
}

impl From<RinexType> for ProductType {
    fn from(rt: RinexType) -> Self {
        match rt {
            RinexType::ObservationData => Self::Observation,
            RinexType::NavigationData => Self::BroadcastNavigation,
            RinexType::MeteoData => Self::MeteoObservation,
            RinexType::ClockData => Self::HighPrecisionClock,
            RinexType::IonosphereMaps => Self::IONEX,
            RinexType::AntennaData => Self::ANTEX,
            RinexType::DORIS => Self::DORIS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetaData {
    /// File name
    pub name: String,
    /// File extension
    pub extension: String,
    /// Product Type
    pub product_id: ProductType,
}

impl MetaData {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();

        let name = path
            .file_stem()
            .ok_or(Error::FileName)?
            .to_string_lossy()
            .to_string();

        let mut extension = path
            .extension()
            .ok_or(Error::FileName)?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            name: if let Some(offset) = name.as_str().find('.') {
                name[..offset].to_string()
            } else {
                name.to_string()
            },
            extension: if let Some(offset) = name.as_str().find('.') {
                extension.insert(0, '.');
                extension.insert_str(0, &name[offset + 1..]);
                extension.to_string()
            } else {
                extension.to_string()
            },
            product_id: Default::default(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::MetaData;

    #[test]
    fn test_meta_data() {
        let path = format!(
            "{}/../test_resources/OBS/V2/aopr0010.17o",
            env!("CARGO_MANIFEST_DIR")
        );

        let meta = MetaData::new(&path).unwrap();

        assert_eq!(meta.name, "aopr0010");
        assert_eq!(meta.extension, "17o");

        let path = format!(
            "{}/../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
            env!("CARGO_MANIFEST_DIR")
        );

        let meta = MetaData::new(&path).unwrap();

        assert_eq!(meta.name, "ESBC00DNK_R_20201770000_01D_30S_MO");
        assert_eq!(meta.extension, "crx.gz");
    }
}
