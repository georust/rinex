//! GNSS post processing context
use thiserror::Error;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use rinex::{
    merge::{Error as RinexMergeError, Merge as RinexMerge},
    prelude::{
        Antenna as RinexAntenna, Epoch, Error as RinexError, GroundPosition, Receiver, Rinex,
        RinexType, DOMES,
    },
};

#[cfg(feature = "sp3")]
use sp3::{prelude::SP3, Merge as SP3Merge, MergeError as SP3MergeError};

use horrorshow::{box_html, helper::doctype, html, RenderBox};
use rinex_qc_traits::HtmlReport;

#[derive(Debug, Error)]
pub enum Error {
    #[error("non supported file format")]
    NonSupportedFileFormat,
    #[error("invalid rinex data")]
    RinexError(#[from] RinexError),
    #[error("cannot index file uniquely")]
    CannotIndexUniquely,
    #[error("failed to extend rinex context")]
    RinexMergeError(#[from] RinexMergeError),
    #[cfg(feature = "sp3")]
    #[error("failed to extend sp3 context")]
    SP3MergeError(#[from] SP3MergeError),
}

/// Station represents GNSS stations or production Agencies.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum StationIndex {
    /// DOMES site number is the best method to identify a station uniquely
    /// (highly recommended).
    DOMES(DOMES),
    /// Agency
    Agency(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum ReceiverIndex {
    /// GNSS receiver
    Receiver(Receiver),
    /// Identify a receiver by the antenna attached to it
    Antenna(String),
}

impl From<&RinexAntenna> for ReceiverIndex {
    fn from(ant: &RinexAntenna) -> Self {
        Self::Antenna(format!("{}-{}", ant.model, ant.sn))
    }
}

/// Defines what makes one file unique in a dataset.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum ProductIndex {
    /// Receiver indexing applies to OBS, DORIS and Meteo RINEX.
    Receiver(ReceiverIndex),
    /// Station BRDC RINEX
    Station(StationIndex),
}

// Wrapper so we can store all files supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd)]
enum FileType {
    /// Meteo RINEX, identified by production Agency.
    MeteoRinex,
    /// Observation RINEX, identified by receiver.
    ObservationRinex,
    /// BRDC RINEX, identified by production Agency.
    BrdcNavigationRinex,
    /// DORIS special RINEX, identified by receiver.
    DorisRinex,
    /// Clock RINEX, identifed by production Agency.
    ClockRinex,
    /// IONEX (special RINEX), identified by Agency.
    IONEX,
    /// ANTEX with single antenna calibration information.
    /// Identifed by reicever.
    ANTEX_CAL,
    /// IGS ANTEX database, is expected to be unique.
    ANTEX_DB,
    /// High Precision SP3 orbits, identified by Agency.
    #[cfg(feature = "sp3")]
    SP3,
}

impl From<RinexType> for FileType {
    fn from(rt: RinexType) -> Self {
        match rt {
            RinexType::DORIS => Self::DorisRinex,
            RinexType::ClockData => Self::ClockRinex,
            RinexType::MeteoData => Self::MeteoRinex,
            RinexType::AntennaData => Self::ANTEX_CAL,
            RinexType::IonosphereMaps => Self::IONEX,
            RinexType::ObservationData => Self::ObservationRinex,
            RinexType::NavigationData => Self::BrdcNavigationRinex,
        }
    }
}

/// RINEX Key is how we identify RINEX files uniquely
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RinexKey {
    /// File type
    pub rinex_type: RinexType,
    /// What makes it unique
    pub index: ProductIndex,
}

/// DataContext represents an dataset we can post process and analyze.
/// Currently supports a single Receiver and does not allow for differential analysis or RTK. Will panic if such scenario is detected.
#[derive(Default)]
pub struct DataContext {
    /// Files contained in Self
    files: HashMap<FileType, Vec<PathBuf>>,
    /// RINEX files contained in Self
    rinex: HashMap<RinexKey, Rinex>,
    #[cfg(feature = "sp3")]
    sp3: Option<SP3>,
}

impl DataContext {
    /// Returns First Epoch (oldest in time) found in Self
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epoch_iter().next()
    }
    /// Returns Last (most recent) Epoch found in Self
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.epoch_iter().last()
    }
    /// Iterate Self one Epoch at a time, in chronological order
    pub fn epoch_iter(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(
            self.rinex_data(RinexType::ObservationData)
                .into_iter()
                .flat_map(|prod| prod.epoch()),
        )
    }
    /// Returns reference to all sorted RINEX of this type
    fn sorted_rinex_data(&self, key: RinexKey) -> Option<&Rinex> {
        self.rinex
            .iter()
            .filter_map(|(k, v)| if *k == key { Some(v) } else { None })
            .reduce(|k, _| k)
    }
    /// Returns reference to all RINEX contained.
    /// TODO: improve this to support multiple stations and receivers.
    fn rinex_data(&self, rinex_type: RinexType) -> Option<&Rinex> {
        self.rinex
            .iter()
            .filter_map(|(k, v)| {
                if k.rinex_type == rinex_type {
                    Some(v)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }
    /// Returns mutable reference to matching RINEX
    fn sorted_rinex_data_mut(&mut self, key: &RinexKey) -> Option<&mut Rinex> {
        self.rinex
            .iter_mut()
            .filter_map(|(k, v)| if k == key { Some(v) } else { None })
            .reduce(|k, _| k)
    }
    /// Returns possible position of a Geodetic Marker.
    /// Returns possible position of a Geodetic Marker.
    /// This will have to be improved if we ever support Differential processing.
    pub fn geodetic_marker_position(&self) -> Option<GroundPosition> {
        if let Some(data) = self.rinex_data(RinexType::ObservationData) {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        if let Some(data) = self.rinex_data(RinexType::NavigationData) {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        None
    }
    /// Loads a single RINEX file into Self.
    pub fn load_rinex(&mut self, path: &Path, rinex: Rinex) -> Result<(), Error> {
        let rinex_type = rinex.header.rinex_type;
        let mut file_type = FileType::from(rinex_type);
        let header = &rinex.header;
        let index = match rinex_type {
            RinexType::ObservationData | RinexType::DORIS => {
                if let Some(rcvr) = &header.rcvr {
                    ProductIndex::Receiver(ReceiverIndex::Receiver(rcvr.clone()))
                } else if let Some(ant) = &header.rcvr_antenna {
                    ProductIndex::Receiver(ReceiverIndex::from(ant))
                } else {
                    return Err(Error::CannotIndexUniquely);
                }
            },
            RinexType::NavigationData => {
                if let Some(attr) = &rinex.prod_attr {
                    ProductIndex::Station(StationIndex::Agency(attr.name.clone()))
                } else {
                    if header.agency.is_empty() {
                        return Err(Error::CannotIndexUniquely);
                    } else {
                        ProductIndex::Station(StationIndex::Agency(header.agency.clone()))
                    }
                }
            },
            RinexType::ClockData => {
                if let Some(clk) = &header.clock {
                    if let Some(domes) = &clk.domes {
                        ProductIndex::Station(StationIndex::DOMES(domes.clone()))
                    } else if let Some(igs) = &clk.igs {
                        ProductIndex::Station(StationIndex::Agency(igs.clone()))
                    } else if let Some(site) = &clk.site {
                        ProductIndex::Station(StationIndex::Agency(site.clone()))
                    } else {
                        return Err(Error::CannotIndexUniquely);
                    }
                } else {
                    return Err(Error::CannotIndexUniquely);
                }
            },
            RinexType::MeteoData => {
                if let Some(attr) = &rinex.prod_attr {
                    ProductIndex::Station(StationIndex::Agency(attr.name.clone()))
                } else {
                    return Err(Error::CannotIndexUniquely);
                }
            },
            RinexType::IonosphereMaps => {
                if let Some(attr) = &rinex.prod_attr {
                    ProductIndex::Station(StationIndex::Agency(attr.name.clone()))
                } else {
                    if header.agency.is_empty() {
                        return Err(Error::CannotIndexUniquely);
                    } else {
                        ProductIndex::Station(StationIndex::Agency(header.agency.clone()))
                    }
                }
            },
            RinexType::AntennaData => {
                if let Some(rcvr) = &header.rcvr {
                    file_type = FileType::ANTEX_CAL;
                    ProductIndex::Receiver(ReceiverIndex::Receiver(rcvr.clone()))
                } else {
                    file_type = FileType::ANTEX_DB;
                    ProductIndex::Station(StationIndex::Agency("IGS".to_string()))
                }
            },
        };
        let key = RinexKey { rinex_type, index };
        if let Some(files) = self
            .files
            .iter_mut()
            .filter_map(|(k, v)| if *k == file_type { Some(v) } else { None })
            .reduce(|k, _| k)
        {
            files.push(path.to_path_buf());
        } else {
            // insert new file type
            self.files.insert(file_type, vec![path.to_path_buf()]);
        }
        if let Some(inner) = self.sorted_rinex_data_mut(&key) {
            // extend existing blob
            inner.merge_mut(&rinex)?;
        } else {
            // insert new data
            self.rinex.insert(key, rinex);
        }
        Ok(())
    }
    /// Loads a single SP3 file into Self.
    #[cfg(feature = "sp3")]
    pub fn load_sp3(&mut self, path: &Path, sp3: SP3) -> Result<(), Error> {
        let file_type = FileType::SP3;
        if let Some(files) = self
            .files
            .iter_mut()
            .filter_map(|(k, v)| if *k == file_type { Some(v) } else { None })
            .reduce(|k, _| k)
        {
            files.push(path.to_path_buf());
        } else {
            // insert new file type
            self.files.insert(file_type, vec![path.to_path_buf()]);
        }
        if let Some(inner) = &mut self.sp3 {
            // extend existing blob
            inner.merge_mut(&sp3)?;
        } else {
            // insert new type
            self.sp3 = Some(sp3);
        }
        Ok(())
    }
}

//     /// Returns path to File considered as Primary in this Context.
//     /// Observation then Navigation files are prefered as Primary files.
//     /// When a unique file had been loaded, it is obviously considered Primary.
//     pub fn primary_path(&self) -> Option<&PathBuf> {
//         /*
//          * Order is important: determines what format are prioritized
//          * in the "primary" determination
//          */
//         for product in [
//             ProductType::Observation,
//             ProductType::BroadcastNavigation,
//             ProductType::MeteoObservation,
//             ProductType::HighPrecisionClock,
//             #[cfg(feature = "sp3")]
//             ProductType::HighPrecisionOrbit,
//             ProductType::IONEX,
//             ProductType::ANTEX,
//         ] {
//             if let Some(paths) = self.files(product) {
//                 /*
//                  * Returns Fist file loaded in this category
//                  */
//                 return paths.first();
//             }
//         }
//         None
//     }
//     /// Returns name of this context.
//     /// Context is named after the file considered as Primary, see [Self::primary_path].
//     /// If no files were previously loaded, simply returns "Undefined".
//     pub fn name(&self) -> String {
//         if let Some(path) = self.primary_path() {
//             path.file_name()
//                 .unwrap_or(OsStr::new("Undefined"))
//                 .to_string_lossy()
//                 // removes possible .crx ; .gz extensions
//                 .split('.')
//                 .next()
//                 .unwrap_or("Undefined")
//                 .to_string()
//         } else {
//             "Undefined".to_string()
//         }
//     }
//     /// Returns reference to files loaded in given category
//     pub fn files(&self, product: ProductType) -> Option<&Vec<PathBuf>> {
//         self.files
//             .iter()
//             .filter_map(|(prod_type, paths)| {
//                 if *prod_type == product {
//                     Some(paths)
//                 } else {
//                     None
//                 }
//             })
//             .reduce(|k, _| k)
//     }
//     /// Returns mutable reference to files loaded in given category
//     pub fn files_mut(&mut self, product: ProductType) -> Option<&Vec<PathBuf>> {
//         self.files
//             .iter()
//             .filter_map(|(prod_type, paths)| {
//                 if *prod_type == product {
//                     Some(paths)
//                 } else {
//                     None
//                 }
//             })
//             .reduce(|k, _| k)
//     }
//     /// Returns reference to inner data of given category
//     fn products_data(&self, product: ProductType) -> &[ProductIndexing, BlobData] {
//         self.blob
//             .iter()
//             .filter_map(|((prod, index), data)| {
//                 if *prod == product {
//                     Some((index, data))
//                 } else {
//                     None
//                 }
//             })
//     }
//     /// Returns mutable reference to inner data of given category
//     fn products_data_mut(&mut self, product: ProductType) -> Option<&mut BlobData> {
//         self.blob
//             .iter_mut()
//             .filter_map(|(prod_type, data)| {
//                 if *prod_type == product {
//                     Some(data)
//                 } else {
//                     None
//                 }
//             })
//             .reduce(move |k, _| k)
//     }
//     /// Returns reference to inner RINEX data of given category
//     pub fn rinex(&self, product: ProductType) -> Option<&Rinex> {
//         self.products_data(product)?.as_rinex()
//     }
//     /// Returns reference to inner SP3 data
//     #[cfg(feature = "sp3")]
//     pub fn sp3(&self) -> Option<&SP3> {
//         self.products_data(ProductType::HighPrecisionOrbit)?.as_sp3()
//     }
//     /// Returns mutable reference to inner RINEX data of given category
//     pub fn rinex_mut(&mut self, product: ProductType) -> Option<&mut Rinex> {
//         self.products_data_mut(product)?.as_mut_rinex()
//     }
//     /// Returns reference to inner [ProductType::Observation] data
//     pub fn observation(&self) -> Option<&Rinex> {
//         self.products_data(ProductType::Observation)?.as_rinex()
//     }
//     /// Returns reference to inner [ProductType::BroadcastNavigation] data
//     pub fn brdc_navigation(&self) -> Option<&Rinex> {
//         self.products_data(ProductType::BroadcastNavigation)?.as_rinex()
//     }
//     /// Returns reference to inner [ProductType::Meteo] data
//     pub fn meteo(&self) -> Option<&Rinex> {
//         self.products_data(ProductType::MeteoObservation)?.as_rinex()
//     }
//     /// Returns reference to inner [ProductType::HighPrecisionClock] data
//     pub fn clock(&self) -> Option<&Rinex> {
//         self.products_data(ProductType::HighPrecisionClock)?.as_rinex()
//     }
//     /// Returns reference to inner [ProductType::ANTEX] data
//     pub fn antex(&self) -> Option<&Rinex> {
//         self.products_data(ProductType::ANTEX)?.as_rinex()
//     }
//     /// Returns reference to all [ProductType::IONEX] products
//     pub fn ionex_products(&self) -> Option<&[Rinex]> {
//         self.products_data(ProductType::IONEX)?.as_rinex()
//     }
//     /// Returns mutable reference to inner [ProductType::Observation] data
//     pub fn observation_mut(&mut self) -> Option<&mut Rinex> {
//         self.products_data_mut(ProductType::Observation)?.as_mut_rinex()
//     }
//     /// Returns mutable reference to inner [ProductType::Observation] data
//     pub fn brdc_navigation_mut(&mut self) -> Option<&mut Rinex> {
//         self.products_data_mut(ProductType::BroadcastNavigation)?
//             .as_mut_rinex()
//     }
//     /// Returns reference to inner [ProductType::Meteo] data
//     pub fn meteo_mut(&mut self) -> Option<&mut Rinex> {
//         self.products_data_mut(ProductType::MeteoObservation)?.as_mut_rinex()
//     }
//     /// Returns mutable reference to inner [ProductType::HighPrecisionClock] data
//     pub fn clock_mut(&mut self) -> Option<&mut Rinex> {
//         self.products_data_mut(ProductType::HighPrecisionClock)?
//             .as_mut_rinex()
//     }
//     /// Returns mutable reference to inner [ProductType::HighPrecisionOrbit] data
//     #[cfg(feature = "sp3")]
//     pub fn sp3_mut(&mut self) -> Option<&mut SP3> {
//         self.products_data_mut(ProductType::HighPrecisionOrbit)?.as_mut_sp3()
//     }
//     /// Returns mutable reference to inner [ProductType::ANTEX] data
//     pub fn antex_mut(&mut self) -> Option<&mut Rinex> {
//         self.products_data_mut(ProductType::ANTEX)?.as_mut_rinex()
//     }
//     /// Returns mutable reference to inner [ProductType::IONEX] data
//     pub fn ionex_mut(&mut self) -> Option<&mut Rinex> {
//         self.products_data_mut(ProductType::IONEX)?.as_mut_rinex()
//     }
//     /// Returns true if [ProductType::Observation] are present in Self
//     pub fn has_observation(&self) -> bool {
//         self.observation().is_some()
//     }
//     /// Returns true if [ProductType::BroadcastNavigation] are present in Self
//     pub fn has_brdc_navigation(&self) -> bool {
//         self.brdc_navigation().is_some()
//     }
//     /// Returns true if [ProductType::HighPrecisionOrbit] are present in Self
//     #[cfg(feature = "sp3")]
//     pub fn has_sp3(&self) -> bool {
//         self.sp3().is_some()
//     }
//     /// Returns true if [ProductType::MeteoObservation] are present in Self
//     pub fn has_meteo(&self) -> bool {
//         self.meteo().is_some()
//     }
//     /// Returns true if High Precision Orbits also contains temporal information.
//     #[cfg(feature = "sp3")]
//     pub fn sp3_has_clock(&self) -> bool {
//         if let Some(sp3) = self.sp3() {
//             sp3.sv_clock().count() > 0
//         } else {
//             false
//         }
//     }
//     /// File revision must be supported and must be correctly formatted
//     /// for this operation to be effective.
//     #[cfg(feature = "sp3")]
//     pub fn load_sp3(&mut self, path: &Path, sp3: SP3) -> Result<(), Error> {
//         let prod_type = ProductType::HighPrecisionOrbit;
//         let indexing = ProductIndexing::Station(StationIndexing::Name(sp3.agency.clone()));
//         // extend file list
//         if let Some(files) = self
//             .files
//             .iter_mut()
//             .filter_map(|(k, files)| if *k == prod_type { Some(files) } else { None })
//             .reduce(|k, _| k)
//         {
//             files.push(path.to_path_buf());
//         } else {
//             // add new file
//             self.files.insert(prod_type, vec![path.to_path_buf()]);
//         }
//
//         // extend possibly existing blob
//         if let Some(paths) = self
//             .blob
//             .iter_mut()
//             .filter_map(|(k, blob)| {
//                 if *k == (prod_type, index) {
//                     Some(blob)
//                 } else {
//                     None
//                 }
//             })
//             .reduce(|k, _| k)
//         {
//             let mut inner = blob.as_mut_sp3().unwrap();
//             inner.merge_mut(&sp3)?;
//         } else {
//         }
//         Ok(())
//     }
// }
//
// impl HtmlReport for DataContext {
//     fn to_html(&self) -> String {
//         format!(
//             "{}",
//             html! {
//                 : doctype::HTML;
//                 html {
//                     head {
//                         meta(charset="UTF-8");
//                         meta(name="viewport", content="width=device-width, initial-scale=1");
//                         link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
//                         script(defer="true", src="https://use.fontawesome.com/releases/v5.3.1/js/all.js");
//                         title: self.name();
//                     }
//                     body {
//                         : self.to_inline_html()
//                     }
//                 }
//             }
//         )
//     }
//     fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
//         box_html! {
//             tr {
//                 th {
//                     : "File"
//                 }
//                 th {
//                     : "Name"
//                 }
//             }
//             @ for product in [
//                 ProductType::Observation,
//                 ProductType::BroadcastNavigation,
//                 ProductType::MeteoObservation,
//                 ProductType::HighPrecisionOrbit,
//                 ProductType::HighPrecisionClock,
//                 ProductType::IONEX,
//                 ProductType::ANTEX,
//             ] {
//                 tr {
//                     td {
//                         : product.to_string()
//                     }
//                     td {
//                         @ if let Some(paths) = self.files(product) {
//                             @ if paths.is_empty() {
//                                 : "None"
//                             } else {
//                                 @ for path in paths {
//                                     br {
//                                         : path.file_name()
//                                             .unwrap()
//                                             .to_string_lossy()
//                                             .to_string()
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
