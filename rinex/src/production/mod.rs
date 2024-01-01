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
use crate::RinexType;
use thiserror::Error;

mod sequence;
pub use sequence::FileSequence;

mod ppu;
pub use ppu::PPU;

mod ffu;
pub use ffu::FFU;

mod src;
pub use src::DataSource;

#[derive(Error, Debug)]
/// File Production errors
pub enum Error {
    #[error("filename does not follow naming conventions")]
    NonStandardFileName,
    #[error("invalid file sequence")]
    InvalidFileSequence,
    #[error("invalid ffu format")]
    InvalidFFU,
}

/// File production attributes. Used when generating
/// RINEX data that follows standard naming conventions,
/// or attached to data parsed from such files.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionAttributes {
    /// Name serves several roles which are type dependent.
    /// - Non detailed OBS RINEX: this is usually the station name
    ///   which can be named after a geodetic marker
    /// - Non detailed NAV RINEX: station name
    /// - CLK RINEX: name of the local clock
    /// - IONEX: agency
    pub name: String,
    /// Year of production
    pub year: u32,
    /// File type dependent specific information
    pub specific: SpecificProductionAttributes,
    /// Detailed production attributes only apply to NAV + OBS RINEX
    /// files. They can only be attached from filenames that follow
    /// the current standardized long format.
    pub details: Option<DetailedProductionAttributes>,
}

#[derive(Debug, Clone, PartialEq)]
/// Type dependent, production attributes.
pub enum SpecificProductionAttributes {
    /// RINEX production attributes.
    Rinex(RinexProductionAttributes),
    /// IONEX production attributes.
    Ionex(IonexProductionAttributes),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct RinexProductionAttributes {
    /// Production Day of Year (DOY)
    pub doy: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct IonexProductionAttributes {
    /// Regional code. 'G' means Global (world wide) TEC map(s).
    pub region: char,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DetailedProductionAttributes {
    /// Agency Country Code
    pub country: String,
    /// Data source
    pub data_src: DataSource,
    /// PPU gives information on file production periodicity.
    pub ppu: PPU,
    /// FFU gives information on Observation sampling rate.
    pub ffu: Option<FFU>,
}

impl ProductionAttributes {
    //pub(crate) fn detailed(&self) -> Option<&DetailedProductionAttributes> {
    //    match self {
    //        Self::Detailed(a) => Some(&a),
    //        _ => None,
    //    }
    //}
    //pub(crate) fn short(&self) -> Option<&ProductionAttributes> {
    //    match self {
    //        Self::Short(a) => Some(&a),
    //        _ => None,
    //    }
    //}
    //pub(crate) fn shorten(&self) -> ProductionAttributes {
    //    match self {
    //        Self::Short(a) => a.clone(),
    //        Self::Detailed(a) => a.shorten(),
    //    }
    //}
}

impl std::str::FromStr for ProductionAttributes {
    type Err = Error;
    fn from_str(fname: &str) -> Result<Self, Self::Err> {
        let fname = fname.to_uppercase();
        if fname.len() < 13 {
            let offset = fname.find('.').unwrap_or(0);
            if offset != 8 {
                return Err(Error::NonStandardFileName);
            };

            // determine type of RINEX first
            // because it determines how to parse the "name" field
            let year = fname[offset + 1..offset + 3]
                .parse::<u32>()
                .map_err(|_| Error::NonStandardFileName)?;

            let rtype = &fname[offset + 3..offset + 4];
            let name_offset = match rtype {
                "I" => 3usize, // only 3 digits on IONEX
                _ => 4usize,
            };

            Ok(Self {
                year: year + 2_000, // year uses 2 digit in old format
                name: fname[..name_offset].to_string(),
                specific: match rtype {
                    "I" => SpecificProductionAttributes::Ionex(IonexProductionAttributes {
                        region: fname.chars().nth(3).unwrap(),
                    }),
                    _ => SpecificProductionAttributes::Rinex(RinexProductionAttributes {
                        doy: fname[4..7]
                            .parse::<u32>()
                            .map_err(|_| Error::NonStandardFileName)?,
                    }),
                },
                details: None,
            })
        } else {
            let offset = fname.find('.').unwrap_or(0);
            if offset < 30 {
                return Err(Error::NonStandardFileName);
            };

            // determine type of RINEX first
            // because it determines how to parse the "name" field
            let year = fname[12..16]
                .parse::<u32>()
                .map_err(|_| Error::NonStandardFileName)?;

            let rtype = &fname[offset + 3..offset + 4];
            let name_offset = match rtype {
                "I" => 3usize, // only 3 digits on IONEX
                _ => 4usize,
            };

            Ok(Self {
                year,
                name: fname[..name_offset].to_string(),
                specific: match rtype {
                    "I" => {
                        /* IONEX files how only use short format */
                        return Err(Error::NonStandardFileName);
                    },
                    _ => SpecificProductionAttributes::Rinex(RinexProductionAttributes {
                        doy: fname[16..19]
                            .parse::<u32>()
                            .map_err(|_| Error::NonStandardFileName)?,
                    }),
                },
                details: Some(DetailedProductionAttributes {
                    country: fname[6..9].to_string(),
                    data_src: DataSource::from_str(&fname[10..11])?,
                    ppu: PPU::from_str(&fname[24..27])?,
                    ffu: match offset {
                        34 => Some(FFU::from_str(&fname[28..32])?),
                        _ => None, // NAV FILE case
                    },
                }),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::DetailedProductionAttributes;
    use super::IonexProductionAttributes;
    use super::ProductionAttributes;
    use super::RinexProductionAttributes;
    use super::SpecificProductionAttributes;
    use super::{DataSource, FFU, PPU};
    use hifitime::Unit;
    use std::str::FromStr;
    #[test]
    fn short_rinex_filenames() {
        for (filename, name, year, specific) in [
            (
                "AJAC3550.21O",
                "AJAC",
                2021,
                RinexProductionAttributes { doy: 355 },
            ),
            (
                "AJAC3550.21D",
                "AJAC",
                2021,
                RinexProductionAttributes { doy: 355 },
            ),
            (
                "KOSG0010.15O",
                "KOSG",
                2015,
                RinexProductionAttributes { doy: 1 },
            ),
            (
                "rovn0010.21o",
                "ROVN",
                2021,
                RinexProductionAttributes { doy: 1 },
            ),
            (
                "barq071q.19o",
                "BARQ",
                2019,
                RinexProductionAttributes { doy: 71 },
            ),
            (
                "VLNS0010.22D",
                "VLNS",
                2022,
                RinexProductionAttributes { doy: 1 },
            ),
        ] {
            println!("Testing RINEX filename \"{}\"", filename);
            let attrs = ProductionAttributes::from_str(filename).unwrap();
            assert_eq!(attrs.name, name);
            assert_eq!(attrs.year, year);
            assert_eq!(
                attrs.specific,
                SpecificProductionAttributes::Rinex(specific)
            );
        }
    }
    #[test]
    fn long_rinex_filenames() {
        for (filename, name, year, specific, detail) in [
            (
                "ACOR00ESP_R_20213550000_01D_30S_MO.crx",
                "ACOR",
                2021,
                RinexProductionAttributes { doy: 355 },
                DetailedProductionAttributes {
                    country: "ESP".to_string(),
                    data_src: DataSource::Receiver,
                    ppu: PPU::Daily,
                    ffu: Some(FFU {
                        val: 30,
                        unit: Unit::Second,
                    }),
                },
            ),
            (
                "KMS300DNK_R_20221591000_01H_30S_MO.crx",
                "KMS3",
                2022,
                RinexProductionAttributes { doy: 159 },
                DetailedProductionAttributes {
                    country: "DNK".to_string(),
                    data_src: DataSource::Receiver,
                    ppu: PPU::Hourly,
                    ffu: Some(FFU {
                        val: 30,
                        unit: Unit::Second,
                    }),
                },
            ),
            (
                "AMEL00NLD_R_20210010000_01D_MN.rnx",
                "AMEL",
                2021,
                RinexProductionAttributes { doy: 1 },
                DetailedProductionAttributes {
                    country: "NLD".to_string(),
                    data_src: DataSource::Receiver,
                    ppu: PPU::Daily,
                    ffu: None,
                },
            ),
            (
                "MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz",
                "MOJN",
                2020,
                RinexProductionAttributes { doy: 177 },
                DetailedProductionAttributes {
                    country: "DNK".to_string(),
                    data_src: DataSource::Receiver,
                    ppu: PPU::Daily,
                    ffu: Some(FFU {
                        val: 30,
                        unit: Unit::Second,
                    }),
                },
            ),
        ] {
            println!("Testing RINEX filename \"{}\"", filename);
            let attrs = ProductionAttributes::from_str(filename).unwrap();
            assert_eq!(attrs.name, name);
            assert_eq!(attrs.year, year);
            assert_eq!(
                attrs.specific,
                SpecificProductionAttributes::Rinex(specific)
            );
            assert_eq!(attrs.details, Some(detail));
        }
    }
    #[test]
    fn ionex_filenames() {
        for (filename, name, year, specific) in [
            (
                "CKMG0020.22I",
                "CKM",
                2022,
                IonexProductionAttributes { region: 'G' },
            ),
            (
                "CKMG0090.21I",
                "CKM",
                2021,
                IonexProductionAttributes { region: 'G' },
            ),
            (
                "jplg0010.17i",
                "JPL",
                2017,
                IonexProductionAttributes { region: 'G' },
            ),
        ] {
            println!("Testing IONEX filename \"{}\"", filename);
            let attrs = ProductionAttributes::from_str(filename).unwrap();
            assert_eq!(attrs.name, name);
            assert_eq!(attrs.year, year);
            assert_eq!(
                attrs.specific,
                SpecificProductionAttributes::Ionex(specific)
            );
        }
    }
}
