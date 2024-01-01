/*
 * File Production infrastructure.
 * File production information are specified in RINEX files that were named
 * according to standard specifications.
 *
 * Two use cases of this module:
 *   1. When a RINEX was parsed succesfully, we attach Self
 *      if we do regocnized a standard name.
 *      This helps regenerating a filename that still follows the standards.
 *      If ProductionAttributes are not recognized, it is not that big of a deal.
 *      It just means it will be difficult to easily regenerate a filename that
 *      strictly follows the standards, because we will then miss some information
 *      like the country code.
 *
 *   2. In our file production API, we can pass ProductionAttributes
 *      to customize the production of this context.
 */
use thiserror::Error;

#[derive(Error, Debug)]
/// File Production errors
pub enum Error {
    #[error("filename does not follow naming conventions")]
    NonStandardFilename,
    #[error("invalid file sequence")]
    InvalidFileSequence,
}

mod rinex;
pub use rinex::{DataSource, RinexProductionAttributes};

mod ionex;
pub use ionex::IonexProductionAttributes;

#[derive(Debug, Clone, PartialEq)]
/// File production attributes. Attached to a RINEX
/// that was parsed from a file named after standard conventions,
/// or used in customized production context.
pub enum ProductionAttributes {
    /// RINEX file production attributes
    Rinex(RinexProductionAttributes),
    /// IONEX file production attributes
    Ionex(IonexProductionAttributes),
}

impl ProductionAttributes {
    pub(crate) fn as_rinex(&self) -> Option<&RinexProductionAttributes> {
        match self {
            Self::Rinex(a) => Some(&a),
            _ => None,
        }
    }
    pub(crate) fn as_ionex(&self) -> Option<&IonexProductionAttributes> {
        match self {
            Self::Ionex(a) => Some(&a),
            _ => None,
        }
    }
}

impl Default for ProductionAttributes {
    fn default() -> Self {
        Self::Rinex(RinexProductionAttributes::default())
    }
}

impl std::str::FromStr for ProductionAttributes {
    type Err = Error;
    fn from_str(fname: &str) -> Result<Self, Self::Err> {
        if let Ok(rinex) = RinexProductionAttributes::from_str(fname) {
            Ok(Self::Rinex(rinex))
        } else if let Ok(ionex) = IonexProductionAttributes::from_str(fname) {
            Ok(Self::Ionex(ionex))
        } else {
            Err(Error::NonStandardFilename)
        }
    }
}

#[cfg(test)]
mod test {
    use super::DataSource;
    use super::ProductionAttributes;
    use std::str::FromStr;
    #[test]
    fn prod_attributes() {
        for filename in [
            "ACOR00ESP_R_20213550000_01D_30S_MO",
            "ESBC00DNK_R_20201770000_01D_30S_MO",
            "CKMG0020.22I",
            "CKMG0090.21I.gz",
            "jplg0010.17i.gz",
        ] {
            let attrs = ProductionAttributes::from_str(filename);
            assert!(attrs.is_ok());
            // let attrs = attrs.unwrap();
            // assert_eq!(attrs.country, country);
            // assert_eq!(attrs.station, station);
            // assert_eq!(attrs.data_src, data_src);
        }
    }
}
